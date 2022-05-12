$env:RUST_LOG="info"
Get-Content input.txt | cargo run -- --pattern '<title>(.*)</title>'