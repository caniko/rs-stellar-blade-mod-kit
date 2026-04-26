#include "bouldy_adapter.hpp"
#include "recon_database.hpp"

#include <cassert>
#include <filesystem>
#include <iostream>
#include <sqlite3.h>
#include <vector>

namespace {

int count_rows(sqlite3* db, const char* table) {
    std::string sql = "SELECT COUNT(*) FROM ";
    sql += table;
    sqlite3_stmt* stmt = nullptr;
    assert(sqlite3_prepare_v2(db, sql.c_str(), -1, &stmt, nullptr) == SQLITE_OK);
    assert(sqlite3_step(stmt) == SQLITE_ROW);
    const int count = sqlite3_column_int(stmt, 0);
    sqlite3_finalize(stmt);
    return count;
}

extern "C" bool visitor(const bouldy::recon::DiscoveryCandidateV2* candidate, void* user_data) {
    assert(candidate != nullptr);
    auto* visits = static_cast<int*>(user_data);
    ++(*visits);
    assert(candidate->schema_version == bouldy::recon::DiscoveryCandidateSchemaVersion);
    assert(candidate->object_index == 12);
    return true;
}

} // namespace

int main() {
    using namespace bouldy::recon;

    const char* terms[] = {"parry", "dodge"};
    DiscoveryFilter filter{terms, 2, DiscoveryKindFunction | DiscoveryKindProperty, 8};

    CandidateRecord parry{
        DiscoveryKindFunction,
        "CanPerfectParry",
        "/Script/SB.PlayerCombatComponent",
        "Eve",
        7,
        2,
        12,
    };
    CandidateRecord weather{
        DiscoveryKindProperty,
        "WeatherIntensity",
        "/Script/SB.WorldLighting",
        "Environment",
        0,
        -1,
        -1,
    };

    assert(candidate_matches_filter(parry, &filter));
    assert(!candidate_matches_filter(weather, &filter));

    const std::filesystem::path db_path =
        std::filesystem::temp_directory_path() / "bouldy_recon_tests.sqlite3";
    std::filesystem::remove(db_path);

    ReconDatabase database;
    assert(database.open(db_path));
    assert(database.start_scan_run());

    int visits = 0;
    const std::vector<CandidateRecord> candidates{parry, weather};
    const ScanSummary summary =
        visit_candidates(candidates, &filter, &visitor, &visits, &database);
    assert(summary.seen == 2);
    assert(summary.matched == 1);
    assert(summary.visited == 1);
    assert(visits == 1);

    assert(database.insert_export("stellar_blade.combat_recon", "{\"kind\":4}"));
    assert(count_rows(database.handle(), "scan_runs") == 1);
    assert(count_rows(database.handle(), "candidates") == 2);
    assert(count_rows(database.handle(), "exports") == 1);

    std::filesystem::remove(db_path);
    std::cout << "bouldy_recon_tests passed\n";
    return 0;
}
