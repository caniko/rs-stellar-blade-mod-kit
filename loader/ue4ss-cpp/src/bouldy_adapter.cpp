#include "bouldy_adapter.hpp"

#include "recon_database.hpp"

#include <algorithm>
#include <cctype>

namespace bouldy::recon {
namespace {

std::string lowercase(std::string_view value) {
    std::string lowered;
    lowered.reserve(value.size());
    for (unsigned char ch : value) {
        lowered.push_back(static_cast<char>(std::tolower(ch)));
    }
    return lowered;
}

bool contains_term(std::string_view haystack, const std::vector<std::string>& terms) {
    const std::string lowered_haystack = lowercase(haystack);
    return std::any_of(terms.begin(), terms.end(), [&](const std::string& term) {
        return !term.empty() && lowered_haystack.find(term) != std::string::npos;
    });
}

bool kind_allowed(std::uint32_t candidate_kind, const DiscoveryFilter* filter) {
    if (!filter || filter->kind_mask == 0) {
        return true;
    }
    return (candidate_kind & filter->kind_mask) != 0;
}

} // namespace

std::vector<std::string> filter_terms(const DiscoveryFilter* filter) {
    std::vector<std::string> terms;
    if (!filter || !filter->terms) {
        return terms;
    }

    terms.reserve(filter->term_count);
    for (std::size_t index = 0; index < filter->term_count; ++index) {
        const char* term = filter->terms[index];
        if (term && term[0] != '\0') {
            terms.push_back(lowercase(term));
        }
    }
    return terms;
}

bool candidate_matches_filter(const CandidateRecord& candidate, const DiscoveryFilter* filter) {
    if (!kind_allowed(candidate.kind, filter)) {
        return false;
    }

    const std::vector<std::string> terms = filter_terms(filter);
    if (terms.empty()) {
        return true;
    }

    return contains_term(candidate.name, terms) || contains_term(candidate.path, terms)
        || contains_term(candidate.owner, terms);
}

DiscoveryCandidateV2 to_abi_candidate(const CandidateRecord& candidate) {
    return DiscoveryCandidateV2{
        candidate.kind,
        DiscoveryCandidateSchemaVersion,
        candidate.name.empty() ? nullptr : candidate.name.c_str(),
        candidate.path.empty() ? nullptr : candidate.path.c_str(),
        candidate.owner.empty() ? nullptr : candidate.owner.c_str(),
        candidate.flags,
        candidate.chunk_index,
        candidate.object_index,
    };
}

ScanSummary visit_candidates(const std::vector<CandidateRecord>& candidates,
                             const DiscoveryFilter* filter,
                             DiscoveryVisitorV2 visitor,
                             void* user_data,
                             ReconDatabase* database) {
    ScanSummary summary{};
    const std::size_t max_results = filter ? filter->max_results : 0;

    for (const CandidateRecord& candidate : candidates) {
        ++summary.seen;
        if (database) {
            database->insert_candidate(candidate);
        }

        if (!candidate_matches_filter(candidate, filter)) {
            continue;
        }

        ++summary.matched;
        if (!visitor) {
            continue;
        }

        const DiscoveryCandidateV2 abi_candidate = to_abi_candidate(candidate);
        if (!visitor(&abi_candidate, user_data)) {
            break;
        }

        ++summary.visited;
        if (max_results != 0 && summary.visited >= max_results) {
            break;
        }
    }

    return summary;
}

std::filesystem::path default_database_path() {
    return std::filesystem::path{"Mods"} / "StellarBladeBouldyRecon" / "data"
        / "recon.sqlite3";
}

} // namespace bouldy::recon
