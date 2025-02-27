cmake_minimum_required(VERSION 3.20)
project(otter_sat_IPASIR C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# set(CMAKE_C_COMPILER gcc)
# set(CMAKE_C_STANDARD 11)
# set(CMAKE_C_STANDARD_REQUIRED TRUE)
# set(CMAKE_C_FLAGS "-Wall -DNDEBUG -O3")

# set(CMAKE_CXX_COMPILER g++)
# set(CMAKE_CXX_FLAGS "-Wall -DNDEBUG -O0 -g")

# genipafolio requires random_shuffle, depreciated in 14+
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)

include_directories(${CMAKE_SOURCE_DIR}/../../target/release/)

message("Calling cargo")
execute_process(COMMAND "cargo" "rustc" "-r" "--lib" "--crate-type=cdylib" WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/../../otter_sat")

if (NOT EXISTS "${CMAKE_CURRENT_BINARY_DIR}/minisat220")
  message("Building minisat220")
  file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/minisat220")
  file(COPY "${CMAKE_SOURCE_DIR}/ipasir/sat/minisat220" DESTINATION "${CMAKE_CURRENT_BINARY_DIR}" FOLLOW_SYMLINK_CHAIN)
  execute_process(COMMAND "make" WORKING_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/minisat220")
endif()


file(GLOB LIBRARIES "../../target/release/*.dylib")
message("Libraries = ${LIBRARIES}")

# genipabones

add_executable(genipabones_otter_sat ipasir/app/genipabones/genipabones.cpp)
target_link_libraries(genipabones_otter_sat ${LIBRARIES})

add_executable(genipabones_minisat ipasir/app/genipabones/genipabones.cpp)
target_link_libraries(genipabones_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# genipaessentials

add_executable(genipaessentials_otter_sat ipasir/app/genipaessentials/genipaessentials.cpp)
target_link_libraries(genipaessentials_otter_sat ${LIBRARIES})

add_executable(genipaessentials_minisat ipasir/app/genipaessentials/genipaessentials.cpp)
target_link_libraries(genipaessentials_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# genipafolio

add_executable(genipafolio_otter_sat ipasir/app/genipafolio/genipafolio.cpp)
target_link_libraries(genipafolio_otter_sat ${LIBRARIES})

add_executable(genipafolio_minisat ipasir/app/genipafolio/genipafolio.cpp)
target_link_libraries(genipafolio_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# genipalsp

add_executable(genipalsp_otter_sat ipasir/app/genipalsp/genipalsp.cpp)
target_link_libraries(genipalsp_otter_sat ${LIBRARIES})

add_executable(genipalsp_minisat ipasir/app/genipalsp/genipalsp.cpp)
target_link_libraries(genipalsp_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# genipareach

add_executable(genipareach_otter_sat ipasir/app/genipareach/genipareach.cpp)
target_link_libraries(genipareach_otter_sat ${LIBRARIES})

add_executable(genipareach_minisat ipasir/app/genipareach/genipareach.cpp)
target_link_libraries(genipareach_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# ipasir-check-conflict

add_executable(ipasir-check-conflict_otter_sat ipasir/app/ipasir-check-conflict/ipasir-check-conflict.cc)
target_link_libraries(ipasir-check-conflict_otter_sat ${LIBRARIES})

add_executable(ipasir-check-conflict_minisat ipasir/app/ipasir-check-conflict/ipasir-check-conflict.cc)
target_link_libraries(ipasir-check-conflict_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# ipasir-check-iterative

add_executable(ipasir-check-iterative_otter_sat ipasir/app/ipasir-check-iterative/ipasir-check-iterative.cc)
target_link_libraries(ipasir-check-iterative_otter_sat ${LIBRARIES})

add_executable(ipasir-check-iterative_minisat ipasir/app/ipasir-check-iterative/ipasir-check-iterative.cc)
target_link_libraries(ipasir-check-iterative_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)

# ipasir-check-satunsat

add_executable(ipasir-check-satunsat_otter_sat ipasir/app/ipasir-check-satunsat/ipasir-check-satunsat.cc)
target_link_libraries(ipasir-check-satunsat_otter_sat ${LIBRARIES})

add_executable(ipasir-check-satunsat_minisat ipasir/app/ipasir-check-satunsat/ipasir-check-satunsat.cc)
target_link_libraries(ipasir-check-satunsat_minisat ${CMAKE_CURRENT_BINARY_DIR}/minisat220/libipasirminisat220.a)
