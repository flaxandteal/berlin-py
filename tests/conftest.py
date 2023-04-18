import pytest
from typing import List
from unittest.mock import patch
from dataclasses import dataclass

from pathlib import Path
from csv import DictReader
import json
from berlin import load_from_json
from berlin._utils import CSV_NAME_MAP

TEST_DATA_DIR = Path(__file__).parent / "data"

def load_test_codes():
    with (TEST_DATA_DIR / "test-codes.json").open() as jsonf:
        return jsonf.read()

def load_test_code_list():
    with (TEST_DATA_DIR / "test-code-list.csv").open() as csvf:
        return [
            {CSV_NAME_MAP[k]: v for k, v in row.items() if k in CSV_NAME_MAP}
            for row in DictReader(csvf)
        ]

@pytest.fixture()
def db():
    return load_from_json([[load_test_codes()]], load_test_code_list())
