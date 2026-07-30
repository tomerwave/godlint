import hashlib

etag = hashlib.md5(body).hexdigest()
