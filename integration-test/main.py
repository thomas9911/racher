import requests
import functools

URL = "http://127.0.0.1:9226"


class RacherRequest:
    def __init__(self, url=URL):
        self.url = url

    def _do(self, cmd, name=None, *args, **kwargs):
        if name:
            r = requests.post(f"{self.url}/{cmd}/{name}", *args, **kwargs)
        else:
            r = requests.post(f"{self.url}/{cmd}", *args, **kwargs)

        if r.ok:
            return r.json()
        else:
            raise Exception(f"{r.text}:{r.status_code}")

    def setter(self, name, data):
        return self._do("set", name, json=data)

    def getter(self, name):
        return self._do("get", name)

    def delete(self, name):
        return self._do("del", name)

    def fetch_keys(self):
        return self._do("keys")

    def purge(self):
        return self._do("purge")


class Racher(RacherRequest):
    def get(self, key):
        value = self.getter(key)["data"]
        if value:
            return value["1"]
        return None

    def set_item(self, key, value):
        return self.setter(key, {"1": value})

    def __getitem__(self, key):
        return self.get(key)

    def __setitem__(self, key, value):
        return self.set_item(key, value)

    def __delitem__(self, key):
        self.delete(key)

    def keys(self):
        return self.fetch_keys()["keys"]


def cache(cache):
    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            joined_args = "".join([f"::{x}" for x in args])
            joined_kwargs = "".join([f"::{x}={kwargs[x]}" for x in sorted(kwargs)])
            cache_key = f"{func.__name__}{joined_args}{joined_kwargs}"
            value = cache.get(cache_key)
            if value:
                return value

            value = func(*args, **kwargs)
            cache.set_item(cache_key, value)
            return value

        return wrapper

    return decorator

def tester():
    print("------- dict ------------------")

    c = Racher()
    c["potato"] = {"test": 1}
    print(c["potato"])
    del c["potato"]
    print(c["potato"])

    print(c.keys())


    @cache(c)
    def factorial(n):
        if n < 2:
            return 1
        return n * factorial(n - 1)


    @cache(c)
    def fibo(number):
        if number == 0:
            return 0
        elif number == 1:
            return 1
        return fibo(number - 1) + fibo(number - 2)


    print("------- factorial ------------------")
    print(factorial(160))


    print(c["factorial::5"])
    print(c["factorial::160"])

    del c["factorial::160"]

    print(c["factorial::160"])


    for i in range(1200):
        fibo(i)


    print("fibonaci number 1200", fibo(1200))

    # print(c.purge())


if __name__ == "__main__":
    # tester()
    c = Racher()
    c.setter("test%2Fhorse%2Fkey", {"test": 1234})

    # print(c._do("join", json={"host": "127.0.0.1:1235"}))
    # a  = c._do("_internal/join", json={"host": "127.0.0.1:1235"})
    # print(c._do("_internal/sync", json=a))
