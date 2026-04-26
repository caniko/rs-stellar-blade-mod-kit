#include "bouldy_adapter.hpp"
#include "recon_database.hpp"

#include <windows.h>

#include <cstdio>
#include <mutex>
#include <vector>

extern "C" {
struct UnrealApi {
    void (*log)(const char* message);
    float (*get_delta_seconds)();
};

using TickCallback = void (*)(float delta_seconds);
using ShutdownCallback = void (*)();

struct UnrealApiV1 {
    UnrealApi base;
    void (*register_tick)(TickCallback callback);
    void (*register_shutdown)(ShutdownCallback callback);
};

struct UnrealDiscoveryApiV2 {
    size_t (*scan)(const bouldy::recon::DiscoveryFilter* filter,
                   bouldy::recon::DiscoveryVisitorV2 visitor,
                   void* user_data);
    void (*export_record)(const char* channel, const char* payload);
};

struct UnrealApiV3 {
    UnrealApiV1 lifecycle;
    UnrealDiscoveryApiV2 discovery;
};

using BouldyInitV3 = bool (*)(UnrealApiV3* api);
} // extern "C"

namespace {

HMODULE g_rust_module = nullptr;
TickCallback g_tick_callback = nullptr;
ShutdownCallback g_shutdown_callback = nullptr;
float g_last_delta_seconds = 0.0f;
std::mutex g_database_mutex;
bouldy::recon::ReconDatabase g_database;

void shim_log(const char* message) {
    std::printf("[StellarBladeBouldyRecon] %s\n", message ? message : "<null>");
}

float get_delta_seconds() {
    return g_last_delta_seconds;
}

void register_tick(TickCallback callback) {
    g_tick_callback = callback;
}

void register_shutdown(ShutdownCallback callback) {
    g_shutdown_callback = callback;
}

std::vector<bouldy::recon::CandidateRecord> collect_unreal_candidates() {
    std::vector<bouldy::recon::CandidateRecord> candidates;

#ifdef BOULDY_WITH_UE4SS
    // Wire this function to the pinned UE4SSCPPTemplate/UEPseudo headers.
    // The implementation should iterate GUObjectArray/ForEachUObject-equivalent
    // objects, classify UObject/UClass/UFunction/FProperty-like records, and
    // fill CandidateRecord without reading property values.
#endif

    return candidates;
}

size_t scan_discovery(const bouldy::recon::DiscoveryFilter* filter,
                      bouldy::recon::DiscoveryVisitorV2 visitor,
                      void* user_data) {
    std::lock_guard lock{g_database_mutex};
    const std::vector<bouldy::recon::CandidateRecord> candidates = collect_unreal_candidates();
    const bouldy::recon::ScanSummary summary =
        bouldy::recon::visit_candidates(candidates, filter, visitor, user_data, &g_database);

    char message[256]{};
    std::snprintf(message,
                  sizeof(message),
                  "scan seen=%zu matched=%zu visited=%zu db=%s",
                  summary.seen,
                  summary.matched,
                  summary.visited,
                  g_database.path().string().c_str());
    shim_log(message);
    return summary.visited;
}

void export_record(const char* channel, const char* payload) {
    std::lock_guard lock{g_database_mutex};
    if (!g_database.insert_export(channel, payload)) {
        shim_log("failed to insert export record");
    }
}

} // namespace

bool Startup() {
    if (!g_database.open(bouldy::recon::default_database_path())
        || !g_database.start_scan_run()) {
        shim_log("failed to initialize recon.sqlite3");
        return false;
    }

    g_rust_module = LoadLibraryW(L"stellar_blade_bouldy_mod.dll");
    if (!g_rust_module) {
        shim_log("failed to load stellar_blade_bouldy_mod.dll");
        return false;
    }

    auto init = reinterpret_cast<BouldyInitV3>(
        GetProcAddress(g_rust_module, "bouldy_rust_init_v3"));
    if (!init) {
        shim_log("failed to find bouldy_rust_init_v3");
        FreeLibrary(g_rust_module);
        g_rust_module = nullptr;
        return false;
    }

    UnrealApiV3 api{};
    api.lifecycle.base.log = &shim_log;
    api.lifecycle.base.get_delta_seconds = &get_delta_seconds;
    api.lifecycle.register_tick = &register_tick;
    api.lifecycle.register_shutdown = &register_shutdown;
    api.discovery.scan = &scan_discovery;
    api.discovery.export_record = &export_record;

    if (!init(&api)) {
        shim_log("Rust mod initialization returned false");
        FreeLibrary(g_rust_module);
        g_rust_module = nullptr;
        return false;
    }

    return true;
}

void OnTick(float delta_seconds) {
    g_last_delta_seconds = delta_seconds;
    if (g_tick_callback) {
        g_tick_callback(delta_seconds);
    }
}

void Shutdown() {
    if (g_shutdown_callback) {
        g_shutdown_callback();
        g_shutdown_callback = nullptr;
    }

    g_tick_callback = nullptr;

    if (g_rust_module) {
        FreeLibrary(g_rust_module);
        g_rust_module = nullptr;
    }
}
