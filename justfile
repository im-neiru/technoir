[working-directory("./plugins")]
new-moon world project_name:
    wit-bindgen moonbit ../wit --world "{{ world }}" --out-dir "./{{ project_name }}" --project-name "im-neiru/{{ project_name }}" --derive-eq --derive-debug

[working-directory("./plugins")]
moon-sdk:
    wit-bindgen moonbit ../wit --world "api" --out-dir "moonbit_sdk" --project-name "im-neiru/technoir" --derive-eq --derive-debug
