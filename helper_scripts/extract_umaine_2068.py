#!/usr/bin/env python3
"""Run the config-backed extractor for UMaine Bulletin 2068"""

import sys
from pathlib import Path

from extract_from_config import main as config_main


CONFIG = Path(__file__).with_name("extraction_configs") / "umaine_2068_peaches.json"


if __name__ == "__main__":
    raise SystemExit(config_main([sys.argv[0], str(CONFIG)]))
