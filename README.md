# kzg

A library for [KZG commitments](http://cacr.uwaterloo.ca/techreports/2010/cacr2010-10.pdf) over BLS12-381 in Rust.

## Notes

Uses [blst](https://github.com/supranational/blst) for the curve operations.

WARNING: has not been audited/reviewed for security. Do NOT use in production.

## Features

- [x] KZG setup
- [x] Commit to a polynomial
- [x] Open a commitment
- [ ] Verify an opening