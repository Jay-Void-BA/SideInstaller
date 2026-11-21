"""Compatibility entry point for the archived rootshell checks."""
from make_signing_fixture import create
import sys

if __name__ == "__main__":
    create(sys.argv[1], '--large-directory' in sys.argv[2:])
