# set windows-shell := ["powershell.exe", "-c"]

core-project := "sepa-xml-editor-core"
spa-project := "sepa-xml-editor-spa"

default:
    just --list

build: setup
    cd {{core-project}}; wasm-pack build --release --target web
    rm -rf "{{spa-project}}/external/{{core-project}}"
    cp -r "{{core-project}}/pkg" "{{spa-project}}/external/{{core-project}}"
    cd {{spa-project}}; npm run build

# format:
#     cargo fmt

# lint:
#     cargo clippy -- -W clippy::pedantic

run: build
    cd {{spa-project}}; npm run dev

setup:
    cargo install wasm-pack
    cd {{spa-project}}; npm install