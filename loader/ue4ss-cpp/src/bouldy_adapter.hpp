#pragma once

#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace bouldy::recon {

constexpr std::uint32_t DiscoveryKindUnknown = 0;
constexpr std::uint32_t DiscoveryKindObject = 1u << 0;
constexpr std::uint32_t DiscoveryKindClass = 1u << 1;
constexpr std::uint32_t DiscoveryKindFunction = 1u << 2;
constexpr std::uint32_t DiscoveryKindProperty = 1u << 3;
constexpr std::uint32_t DiscoveryKindAny =
    DiscoveryKindObject | DiscoveryKindClass | DiscoveryKindFunction | DiscoveryKindProperty;
constexpr std::uint32_t DiscoveryCandidateSchemaVersion = 2;

extern "C" {
struct DiscoveryFilter {
    const char* const* terms;
    std::size_t term_count;
    std::uint32_t kind_mask;
    std::size_t max_results;
};

struct DiscoveryCandidateV2 {
    std::uint32_t kind;
    std::uint32_t schema_version;
    const char* name;
    const char* path;
    const char* owner;
    std::uint64_t flags;
    std::int32_t chunk_index;
    std::int32_t object_index;
};

using DiscoveryVisitorV2 =
    bool (*)(const DiscoveryCandidateV2* candidate, void* user_data);
} // extern "C"

struct CandidateRecord {
    std::uint32_t kind{DiscoveryKindUnknown};
    std::string name;
    std::string path;
    std::string owner;
    std::uint64_t flags{0};
    std::int32_t chunk_index{-1};
    std::int32_t object_index{-1};
};

struct ScanSummary {
    std::size_t seen{0};
    std::size_t matched{0};
    std::size_t visited{0};
};

class ReconDatabase;

std::vector<std::string> filter_terms(const DiscoveryFilter* filter);
bool candidate_matches_filter(const CandidateRecord& candidate, const DiscoveryFilter* filter);
DiscoveryCandidateV2 to_abi_candidate(const CandidateRecord& candidate);

ScanSummary visit_candidates(const std::vector<CandidateRecord>& candidates,
                             const DiscoveryFilter* filter,
                             DiscoveryVisitorV2 visitor,
                             void* user_data,
                             ReconDatabase* database);

std::filesystem::path default_database_path();

} // namespace bouldy::recon
