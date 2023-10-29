#!/usr/bin/env bash
cd $(dirname "$0")
sqlite3 ../notifs.db < db.sql
