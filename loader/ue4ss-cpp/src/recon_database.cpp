#include "recon_database.hpp"

#include <chrono>
#include <ctime>
#include <utility>

namespace bouldy::recon {
namespace {

std::string utc_now() {
    const auto now = std::chrono::system_clock::now();
    const std::time_t now_time = std::chrono::system_clock::to_time_t(now);
    std::tm tm{};
#ifdef _WIN32
    gmtime_s(&tm, &now_time);
#else
    gmtime_r(&now_time, &tm);
#endif
    char buffer[32]{};
    std::strftime(buffer, sizeof(buffer), "%Y-%m-%dT%H:%M:%SZ", &tm);
    return buffer;
}

} // namespace

ReconDatabase::ReconDatabase(ReconDatabase&& other) noexcept {
    *this = std::move(other);
}

ReconDatabase& ReconDatabase::operator=(ReconDatabase&& other) noexcept {
    if (this == &other) {
        return *this;
    }
    if (db_) {
        sqlite3_close(db_);
    }
    db_ = other.db_;
    scan_id_ = other.scan_id_;
    path_ = std::move(other.path_);
    other.db_ = nullptr;
    other.scan_id_ = 0;
    return *this;
}

ReconDatabase::~ReconDatabase() {
    if (db_) {
        sqlite3_close(db_);
    }
}

bool ReconDatabase::open(const std::filesystem::path& path) {
    path_ = path;
    if (path_.has_parent_path()) {
        std::filesystem::create_directories(path_.parent_path());
    }

    if (sqlite3_open(path_.string().c_str(), &db_) != SQLITE_OK) {
        return false;
    }

    return exec("PRAGMA journal_mode=WAL;")
        && exec("CREATE TABLE IF NOT EXISTS scan_runs ("
                "id INTEGER PRIMARY KEY AUTOINCREMENT,"
                "started_at_utc TEXT NOT NULL,"
                "game_name TEXT NOT NULL,"
                "engine_assumption TEXT NOT NULL,"
                "mod_version TEXT NOT NULL,"
                "bouldy_api_version INTEGER NOT NULL"
                ");")
        && exec("CREATE TABLE IF NOT EXISTS candidates ("
                "scan_id INTEGER NOT NULL,"
                "kind INTEGER NOT NULL,"
                "name TEXT,"
                "path TEXT,"
                "owner TEXT,"
                "flags INTEGER NOT NULL,"
                "chunk_index INTEGER,"
                "object_index INTEGER"
                ");")
        && exec("CREATE TABLE IF NOT EXISTS exports ("
                "scan_id INTEGER NOT NULL,"
                "channel TEXT NOT NULL,"
                "payload TEXT NOT NULL,"
                "created_at_utc TEXT NOT NULL"
                ");");
}

bool ReconDatabase::start_scan_run() {
    sqlite3_stmt* stmt = nullptr;
    constexpr const char* sql =
        "INSERT INTO scan_runs "
        "(started_at_utc, game_name, engine_assumption, mod_version, bouldy_api_version) "
        "VALUES (?, 'Stellar Blade', 'UE4.26/UE4.26.2', '0.1.0', 3);";
    if (sqlite3_prepare_v2(db_, sql, -1, &stmt, nullptr) != SQLITE_OK) {
        return false;
    }

    const bool ok = bind_text(stmt, 1, utc_now()) && sqlite3_step(stmt) == SQLITE_DONE;
    sqlite3_finalize(stmt);
    if (!ok) {
        return false;
    }
    scan_id_ = sqlite3_last_insert_rowid(db_);
    return true;
}

bool ReconDatabase::insert_candidate(const CandidateRecord& candidate) {
    sqlite3_stmt* stmt = nullptr;
    constexpr const char* sql =
        "INSERT INTO candidates "
        "(scan_id, kind, name, path, owner, flags, chunk_index, object_index) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?);";
    if (sqlite3_prepare_v2(db_, sql, -1, &stmt, nullptr) != SQLITE_OK) {
        return false;
    }

    sqlite3_bind_int64(stmt, 1, scan_id_);
    sqlite3_bind_int(stmt, 2, static_cast<int>(candidate.kind));
    bind_text(stmt, 3, candidate.name);
    bind_text(stmt, 4, candidate.path);
    bind_text(stmt, 5, candidate.owner);
    sqlite3_bind_int64(stmt, 6, static_cast<sqlite3_int64>(candidate.flags));
    if (candidate.chunk_index >= 0) {
        sqlite3_bind_int(stmt, 7, candidate.chunk_index);
    } else {
        sqlite3_bind_null(stmt, 7);
    }
    if (candidate.object_index >= 0) {
        sqlite3_bind_int(stmt, 8, candidate.object_index);
    } else {
        sqlite3_bind_null(stmt, 8);
    }

    const bool ok = sqlite3_step(stmt) == SQLITE_DONE;
    sqlite3_finalize(stmt);
    return ok;
}

bool ReconDatabase::insert_export(const char* channel, const char* payload) {
    sqlite3_stmt* stmt = nullptr;
    constexpr const char* sql =
        "INSERT INTO exports (scan_id, channel, payload, created_at_utc) VALUES (?, ?, ?, ?);";
    if (sqlite3_prepare_v2(db_, sql, -1, &stmt, nullptr) != SQLITE_OK) {
        return false;
    }

    sqlite3_bind_int64(stmt, 1, scan_id_);
    bind_optional_text(stmt, 2, channel);
    bind_optional_text(stmt, 3, payload);
    bind_text(stmt, 4, utc_now());

    const bool ok = sqlite3_step(stmt) == SQLITE_DONE;
    sqlite3_finalize(stmt);
    return ok;
}

bool ReconDatabase::exec(const char* sql) {
    return sqlite3_exec(db_, sql, nullptr, nullptr, nullptr) == SQLITE_OK;
}

bool ReconDatabase::bind_text(sqlite3_stmt* stmt, int index, const std::string& value) {
    return sqlite3_bind_text(stmt, index, value.c_str(), -1, SQLITE_TRANSIENT) == SQLITE_OK;
}

bool ReconDatabase::bind_optional_text(sqlite3_stmt* stmt, int index, const char* value) {
    if (!value) {
        return sqlite3_bind_text(stmt, index, "", -1, SQLITE_TRANSIENT) == SQLITE_OK;
    }
    return sqlite3_bind_text(stmt, index, value, -1, SQLITE_TRANSIENT) == SQLITE_OK;
}

} // namespace bouldy::recon
