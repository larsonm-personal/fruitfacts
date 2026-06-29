#!/usr/bin/env python3
"""Run the config-backed extractor for UMD EB-2023-0684 apples"""

import sys
from pathlib import Path

from extract_from_config import main as config_main


CONFIG = (
    Path(__file__).with_name("extraction_configs")
    / "umd_eb_2023_0684_apples.json"
)


if __name__ == "__main__":
    raise SystemExit(config_main([sys.argv[0], str(CONFIG)]))
