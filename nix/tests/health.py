import os
import time

from utils import c

start = time.time()
while time.time() < start + 5:
    resp = c.get("/health")
    assert resp.status_code == 200
    assert resp.json() == {"database": True, "cache": True, "email": True}

assert os.system("systemctl stop postgresql.service") == 0
time.sleep(2)

resp = c.get("/health", timeout=5)
assert resp.status_code == 500
assert resp.json() == {"database": False, "cache": True, "email": True}

assert os.system("systemctl start postgresql.service") == 0
assert os.system("systemctl stop redis-academy.service") == 0
time.sleep(2)

resp = c.get("/health", timeout=5)
assert resp.status_code == 500
assert resp.json() == {"database": True, "cache": False, "email": True}

assert os.system("systemctl start redis-academy.service") == 0
assert os.system("systemctl stop postfix.service") == 0
time.sleep(2)

resp = c.get("/health", timeout=5)
assert resp.status_code == 500
assert resp.json() == {"database": True, "cache": True, "email": False}

assert os.system("systemctl start postfix.service") == 0
time.sleep(2)

resp = c.get("/health")
assert resp.status_code == 200
assert resp.json() == {"database": True, "cache": True, "email": True}
