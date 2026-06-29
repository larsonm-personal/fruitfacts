#!/usr/bin/env python3
"""Run the config-backed extractor for UGA B807 bunch grapes"""

import sys
from pathlib import Path

from extract_from_config import main as config_main


CONFIG = Path(__file__).with_name("extraction_configs") / "uga_b807_bunch_grapes.json"


if __name__ == "__main__":
    raise SystemExit(config_main([sys.argv[0], str(CONFIG)]))
