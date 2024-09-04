from urllib.parse import parse_qs, urlparse

from utils import c, create_account

resp = c.get("/auth/oauth/providers")
assert resp.status_code == 200
assert resp.json() == [
    {
        "id": "test",
        "name": "Test OAuth2 Provider",
        "authorize_url": "http://127.0.0.1:8002/oauth2/authorize?response_type=code&client_id=client-id",
    }
]

# create link
resp = c.post(
    "http://127.0.0.1:8002/oauth2/authorize?response_type=code&client_id=client-id&state=test123&redirect_uri=http://localhost/oauth2/callback",
    data={"id": "42", "name": "foo"},
    follow_redirects=False,
)
assert resp.is_redirect
url = urlparse(resp.headers["location"])
query = parse_qs(url.query)
code = query["code"][0]
state = query["state"][0]
assert state == "test123"

create_account("a", "a@a", "a")
resp = c.post(
    "/auth/oauth/links/me",
    json={"provider_id": "test", "code": code, "redirect_uri": "http://localhost/oauth2/callback"},
)
assert resp.status_code == 201
link = resp.json()
assert link == {"id": link["id"], "provider_id": "test", "display_name": "foo"}

# list links
resp = c.get("/auth/oauth/links/me")
assert resp.status_code == 200
assert resp.json() == [link]

# delete link
resp = c.delete(f"/auth/oauth/links/me/{link['id']}")
assert resp.status_code == 200
assert resp.json() is True

resp = c.get("/auth/oauth/links/me")
assert resp.status_code == 200
assert resp.json() == []
