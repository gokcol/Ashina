# Reconfiguration is safe; a source revision mismatch fails rather than silently
# shipping the unpatched Windows thread teardown.
execute_process(COMMAND "${GIT_EXECUTABLE}" apply --check "${PATCH_FILE}"
  WORKING_DIRECTORY "${SOURCE_DIR}" RESULT_VARIABLE can_apply
  OUTPUT_QUIET ERROR_QUIET)
if(can_apply EQUAL 0)
  execute_process(COMMAND "${GIT_EXECUTABLE}" apply "${PATCH_FILE}"
    WORKING_DIRECTORY "${SOURCE_DIR}" COMMAND_ERROR_IS_FATAL ANY)
else()
  execute_process(COMMAND "${GIT_EXECUTABLE}" apply --reverse --check "${PATCH_FILE}"
    WORKING_DIRECTORY "${SOURCE_DIR}" RESULT_VARIABLE already_applied
    OUTPUT_QUIET ERROR_QUIET)
  if(NOT already_applied EQUAL 0)
    message(FATAL_ERROR "The Windows translation cleanup patch does not match the pinned sources.")
  endif()
endif()
