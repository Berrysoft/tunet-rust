cmake_minimum_required(VERSION 3.14)
project(TUNetGui VERSION 0.1.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(PROJECT_DIR "${CMAKE_CURRENT_LIST_DIR}/..")
set(TARGET_DIR "${PROJECT_DIR}/target")

if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
    set(LIB_DIR "${TARGET_DIR}/release")
    set(CARGO_FLAGS "--release")
else()
    set(LIB_DIR "${TARGET_DIR}/debug")
    set(CARGO_FLAGS "")
endif()

add_custom_command(
    OUTPUT __always_rebuild
    COMMAND cmake -E echo
)

add_custom_target(tunet-model
    COMMAND cargo build --lib ${CARGO_FLAGS}
    WORKING_DIRECTORY ${PROJECT_DIR}
    DEPENDS __always_rebuild
)

find_package(Qt6 COMPONENTS Core Gui Widgets REQUIRED)

file(GLOB HEADERS "include/*.hpp")
file(GLOB SOURCES "src/*.cpp")

qt_wrap_cpp(SOURCES ${HEADERS})
add_executable(tunet-gui ${SOURCES})
target_include_directories(tunet-gui
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)
target_link_directories(tunet-gui PUBLIC ${LIB_DIR})
target_link_libraries(tunet-gui PUBLIC Qt6::Widgets tunet_model_native)
add_dependencies(tunet-gui tunet-model)

if(WIN32)
    target_link_libraries(tunet-gui PUBLIC ntdll WS2_32 Bcrypt Crypt32 Ncrypt Secur32 Iphlpapi)
    if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
        set_target_properties(tunet-gui PROPERTIES WIN32_EXECUTABLE ON)
    endif()
endif()
