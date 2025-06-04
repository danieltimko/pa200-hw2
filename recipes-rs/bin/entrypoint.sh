#!/bin/bash

# initialzie database (if needed) and run migrations
diesel setup
diesel migration run

./recipes-rs
