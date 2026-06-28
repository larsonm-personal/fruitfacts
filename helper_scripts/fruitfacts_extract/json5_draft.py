"""JSON5 draft emission helpers"""

import json


def q(value):
    return json.dumps(value, ensure_ascii=True)
