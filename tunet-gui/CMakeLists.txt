cmake_minimum_required(VERSION 3.14)
project(TUNetGui VERSION 0.1.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(CheckIPOSupported)
check_ipo_supported(RESULT ENABLE_IPO OUTPUT output)
if(ENABLE_IPO)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
    message(STATUS "LTO is enabled for Release")
endif()

set(PROJECT_DIR "${CMAKE_CURRENT_LIST_DIR}/..")
set(TARGET_DIR "${PROJECT_DIR}/target")

if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
    set(LIB_DIR "${TARGET_DIR}/release")
    set(CARGO_FLAGS "--release")
else()
    set(LIB_DIR "${TARGET_DIR}/debug")
    set(CARGO_FLAGS "")
endif()

file(GLOB RUST_SOURCES "../**/*.rs")

add_custom_target(tunet-model
    COMMAND cargo build --lib ${CARGO_FLAGS}
    WORKING_DIRECTORY ${PROJECT_DIR}
    DEPENDS ${RUST_SOURCES}
)

find_package(Qt6 COMPONENTS Widgets Charts REQUIRED)

file(GLOB SOURCES "src/*.cpp")

qt_wrap_cpp(SOURCES "include/Model.hpp")
add_executable(tunet-gui ${SOURCES})
target_include_directories(tunet-gui
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)
target_link_directories(tunet-gui PUBLIC ${LIB_DIR})
target_link_libraries(tunet-gui PUBLIC Qt6::Widgets Qt6::Charts tunet_model_native)
add_dependencies(tunet-gui tunet-model)

if(WIN32)
    set(WINDOWS_LIB ${CMAKE_BINARY_DIR}/windows.lib)
    if(NOT EXISTS "${WINDOWS_LIB}")
        file(DOWNLOAD https://github.com/microsoft/windows-rs/raw/master/crates/targets/x86_64_msvc/lib/windows.lib ${WINDOWS_LIB})
    endif()
    target_link_libraries(tunet-gui PRIVATE ntdll ${WINDOWS_LIB})
    if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
        set_target_properties(tunet-gui PROPERTIES WIN32_EXECUTABLE ON)
    endif()
endif()
