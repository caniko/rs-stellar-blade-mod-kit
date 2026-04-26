cmake_minimum_required(VERSION 3.22)

project(StellarBladeBouldyRecon LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(BOULDY_BUILD_UE4SS_ENTRY "Build the UE4SS-facing main.dll entry module" OFF)

add_library(sqlite3_amalgamation STATIC third_party/sqlite/sqlite3.c)
target_include_directories(sqlite3_amalgamation PUBLIC third_party/sqlite)

add_library(bouldy_recon_core STATIC
  src/bouldy_adapter.cpp
  src/recon_database.cpp
)
target_include_directories(bouldy_recon_core PUBLIC src third_party/sqlite)
target_link_libraries(bouldy_recon_core PUBLIC sqlite3_amalgamation)

if(BOULDY_BUILD_UE4SS_ENTRY)
  add_library(StellarBladeBouldyRecon SHARED
    src/ue4ss_entry.cpp
  )
  target_link_libraries(StellarBladeBouldyRecon PRIVATE bouldy_recon_core)
  target_compile_definitions(StellarBladeBouldyRecon PRIVATE BOULDY_WITH_UE4SS=1)
endif()

add_executable(bouldy_recon_tests
  tests/recon_tests.cpp
)
target_link_libraries(bouldy_recon_tests PRIVATE bouldy_recon_core)

enable_testing()
add_test(NAME bouldy_recon_tests COMMAND bouldy_recon_tests)
