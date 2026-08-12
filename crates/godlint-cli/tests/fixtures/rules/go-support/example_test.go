package fixture

import (
    "math/rand"
    "net/http"
    "testing"
    "time"
)

func TestEmpty(t *testing.T) {
}

func TestSkipped(t *testing.T) {
    t.Skip()
    rand.Int()
    time.Sleep(time.Second)
    time.After(0)
    time.After()
    http.Get("https://example.test")
}
