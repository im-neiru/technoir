[working-directory("./moons")]
new-moon world project_name:
    wit-bindgen moonbit ../wit --world "{{ world }}" --out-dir "./{{ project_name }}" --project-name "im-neiru/{{ project_name }}" --derive-eq --derive-debug

[working-directory("./moons")]
moon-sdk:
    wit-bindgen moonbit ../wit --world "api" --out-dir "moonbit-sdk" --project-name "im-neiru/technoir" --derive-eq --derive-debug

regen-moon world project_name:
    wit-bindgen moonbit wit --world "{{ world }}" --out-dir "moons/{{ project_name }}" --project-name "im-neiru/{{ project_name }}" --derive-eq --derive-debug --ignore-module-file --ignore-stub

build-moon package:
    moon build "moons/{{package}}/gen" --target wasm
