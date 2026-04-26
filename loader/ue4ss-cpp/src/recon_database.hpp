#pragma once

#include "bouldy_adapter.hpp"

#include <filesystem>
#include <sqlite3.h>
#include <string>

namespace bouldy::recon {

class ReconDatabase {
public:
    ReconDatabase() = default;
    ReconDatabase(const ReconDatabase&) = delete;
    ReconDatabase& operator=(const ReconDatabase&) = delete;
    ReconDatabase(ReconDatabase&& other) noexcept;
    ReconDatabase& operator=(ReconDatabase&& other) noexcept;
    ~ReconDatabase();

    bool open(const std::filesystem::path& path);
    bool start_scan_run();
    bool insert_candidate(const CandidateRecord& candidate);
    bool insert_export(const char* channel, const char* payload);

    [[nodiscard]] sqlite3* handle() const { return db_; }
    [[nodiscard]] std::int64_t scan_id() const { return scan_id_; }
    [[nodiscard]] const std::filesystem::path& path() const { return path_; }

private:
    bool exec(const char* sql);
    bool bind_text(sqlite3_stmt* stmt, int index, const std::string& value);
    bool bind_optional_text(sqlite3_stmt* stmt, int index, const char* value);

    sqlite3* db_{nullptr};
    std::int64_t scan_id_{0};
    std::filesystem::path path_;
};

} // namespace bouldy::recon
