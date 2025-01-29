cmake_minimum_required(VERSION 3.20)
project(otter_sat_IPASIR)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# genipafolio requires random_shuffle, depreciated in 14+
set(CMAKE_CXX_STANDARD 11)

set(CMAKE_CXX_STANDARD_REQUIRED TRUE)

include_directories(${CMAKE_SOURCE_DIR}/../../target/release/)

message("Calling cargo")
execute_process(COMMAND "cargo" "rustc" "-r" "--crate-type=cdylib" WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/../../otter_sat")

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





# add_executable(picosat961 ipasir/sat/picosat961/ipasirpicosatglue.c)
