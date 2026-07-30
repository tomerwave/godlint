import hashlib
import hmac

signature = hmac.new(key, payload, hashlib.sha256).hexdigest()
