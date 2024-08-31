import os
import subprocess
import time

from utils import make_client, refresh_session, save_auth


def test(c):
    try:
        refresh_session(client=c)
    except:
        return False
    else:
        return True


def assert_sessions(expected):
    status, sessions = subprocess.getstatusoutput(
        "sudo -u postgres psql -t --csv academy <<< 'select id from sessions'"
    )
    assert status == 0
    assert sorted(sessions.split()) == sorted(expected)


a = make_client()
resp = a.post("/auth/users", json={"name": "a", "display_name": "a", "email": "a@a", "password": "a"})
assert resp.status_code == 201
al = resp.json()
save_auth(al, a)

b = make_client()
resp = b.post("/auth/users", json={"name": "b", "display_name": "b", "email": "b@b", "password": "b"})
assert resp.status_code == 201
bl = resp.json()
save_auth(bl, b)


assert test(a)
assert test(b)
assert_sessions([al["session"]["id"], bl["session"]["id"]])

for _ in range(7):
    os.system("date -s '+2min'")
    time.sleep(0.5)
    assert test(a)

os.system("systemctl start academy-task-prune-database")
time.sleep(3)

assert_sessions([al["session"]["id"]])
assert test(a)
assert not test(b)
assert_sessions([al["session"]["id"]])
