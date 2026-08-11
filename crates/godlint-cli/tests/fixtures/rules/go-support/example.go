package fixture

import (
    "fmt"
    "os"
    "crypto/md5"
    "math/rand"
    "time"
    "net/http"
    "os/exec"
    "github.com/acme/legacy"
)

// TODO: replace this commented-out code
// fmt.Println("disabled")
func Empty() {}

func Big(a int, b int, c int, d int) (int, int) {
    fmt.Println("a deliberately long repeated literal for duplicate detection")
    fmt.Println("a deliberately long repeated literal for duplicate detection")
    os.Getenv("SECRET")
    md5.New()
    rand.Int()
    time.Sleep(time.Second)
    http.Get("https://example.test")
    exec.Command("sh", "-c", "echo hi")
    os.Exit(1)
    if a > 0 && b > 0 || c > 0 {
        if d > 0 {
            return a, b
        }
    }
    _ = legacy.Legacy
    return c, d
}

// godlint-ignore-next-line maintainability/empty-function
var unused = 1
