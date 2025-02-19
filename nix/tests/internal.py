import subprocess

from utils import c

assert subprocess.getstatusoutput("academy migrate demo --force")[0] == 0

status, jwt = subprocess.getstatusoutput('academy jwt sign \'{"aud":"auth"}\'')
assert status == 0
jwt = jwt.strip()
c.headers["Authorization"] = jwt

FOO = {
    "id": "a8d95e0f-71ae-4c49-995e-695b7c93848c",
    "name": "foo",
    "display_name": "Foo 42",
    "email": "foo@example.com",
    "email_verified": True,
    "registration": 1710423462,
    "last_login": 1710509820,
    "last_name_change": 1710424200,
    "enabled": True,
    "admin": False,
    "password": True,
    "mfa_enabled": False,
    "description": "blubb",
    "tags": ["foo", "bar", "baz"],
    "newsletter": True,
    "business": True,
    "first_name": "x",
    "last_name": "y",
    "street": "asdf",
    "zip_code": "1234",
    "city": "xyz",
    "country": "asdf",
    "vat_id": "1234",
    "can_buy_coins": True,
    "can_receive_coins": True,
    "avatar_url": "https://gravatar.com/avatar/321ba197033e81286fedb719d60d4ed5cecaed170733cb4a92013811afc0e3b6",
}

resp = c.get("/auth/_internal/users/a8d95e0f-71ae-4c49-995e-695b7c93848c")
assert resp.status_code == 200
assert resp.json() == FOO

resp = c.get("/auth/_internal/users/85bae8d0-5419-48ba-9018-88df147a0eb2")
assert resp.status_code == 404
assert resp.json() == {"detail": "User not found"}

resp = c.get("/auth/_internal/users/by_email/Foo@example.com")
assert resp.status_code == 200
assert resp.json() == FOO

resp = c.get("/auth/_internal/users/by_email/not@found")
assert resp.status_code == 404
assert resp.json() == {"detail": "User not found"}

c.headers["Authorization"] = "blubb"
resp = c.get("/auth/_internal/users/a8d95e0f-71ae-4c49-995e-695b7c93848c")
assert resp.status_code == 401
assert resp.json() == {"detail": "Invalid token"}
