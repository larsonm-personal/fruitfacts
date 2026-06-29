#!/usr/bin/env python3
"""Run the config-backed extractor for UNL G2354 fruit tree cultivars"""

import sys
from pathlib import Path

from extract_from_config import main as config_main


CONFIG = (
    Path(__file__).with_name("extraction_configs")
    / "unl_g2354_fruit_tree_cultivars.json"
)


if __name__ == "__main__":
    raise SystemExit(config_main([sys.argv[0], str(CONFIG)]))
