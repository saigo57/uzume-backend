#!/bin/bash

rm ./licenses-list.csv
rm -rf ./tmp-licenses
go-licenses csv . > licenses-list.csv
go-licenses save . --save_path tmp-licenses
ruby generate_notice.rb
