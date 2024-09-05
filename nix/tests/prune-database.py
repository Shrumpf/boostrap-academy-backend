import os
import subprocess
import time

from utils import create_account, make_client, refresh_session


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
al = create_account("a", "a@a", "a", a)

b = make_client()
bl = create_account("b", "b@b", "b", b)


assert test(a)
assert test(b)
assert_sessions([al["session"]["id"], bl["session"]["id"]])

for _ in range(5):
    os.system("date -s '+10days'")
    time.sleep(0.5)
    assert test(a)

os.system("systemctl start academy-task-prune-database.service")
time.sleep(1)

assert_sessions([al["session"]["id"]])
assert test(a)
assert not test(b)
assert_sessions([al["session"]["id"]])
