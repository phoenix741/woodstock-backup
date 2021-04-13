set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 20)

find_package(protobuf CONFIG REQUIRED)
find_package(Qt5 COMPONENTS Core REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(TCMALLOC REQUIRED IMPORTED_TARGET libtcmalloc_minimal)

include_directories(${Protobuf_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories(../../lib)
include_directories(${CMAKE_CURRENT_BINARY_DIR}/../../lib)
include_directories(${TCMALLOC_INCLUDE_DIRS})

file(GLOB_RECURSE SRC CONFIGURE_DEPENDS "*.cpp")

# Create the executable

add_executable(readindex ${SRC})
target_link_libraries(readindex PRIVATE Qt5::Core protobuf::libprotobuf-lite woodstock PkgConfig::TCMALLOC)
