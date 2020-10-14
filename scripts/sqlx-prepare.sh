#!/bin/sh
(cd pt_server && cargo sqlx prepare -- --lib)