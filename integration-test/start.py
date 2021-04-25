BINARY = "D:\\Rust\\racher\\target\\release\\racher.exe"

PORTS = list(range(9226, 9231))
HOST_LIST = [f"127.0.0.1:{x}" for x in PORTS]

import subprocess
import time
import os

# import asyncio

# def run(cmd):
#     proc = asyncio.create_subprocess_shell(
#         cmd,
#         stdout=asyncio.subprocess.PIPE,
#         stderr=asyncio.subprocess.PIPE)
#     return proc

#     # proc = asyncio.create_subprocess_shell(cmd)

#     # print(dir(proc))
#     # proc = await proc

#     # stdout, stderr = await proc.communicate()

#     # print(f'[{cmd!r} exited with {proc.returncode}]')
#     # if stdout:
#     #     print(f'[stdout]\n{stdout.decode()}')
#     # if stderr:
#     #     print(f'[stderr]\n{stderr.decode()}')

# async def main():
#     first = asyncio.create_subprocess_shell(f'{BINARY} -a {HOST_LIST[0]}')
#     await asyncio.sleep(5)

#     #  asyncio.gather(
#     #     *[
#     #         run(f'{BINARY} join -a {x} -j {HOST_LIST[0]}') for x in HOST_LIST[1:]
#     #     ]
#     # )
#     a = [
#             asyncio.create_subprocess_shell(f'{BINARY} join -a {x} -j http://{HOST_LIST[0]}') for x in HOST_LIST[1:]
#         ]

#     proc = await first
#     # stdout, _stderr = await proc.communicate()
#     # print(stdout)
#     print(proc)

#     for coro in asyncio.as_completed(a):
#         proc = await coro
#         stdout, _stderr = await proc.communicate()
#         print(stdout)

#     print("done?")

# asyncio.run(main())


def run(cmd):
    os.environ["RACHER_LOGGER_LEVEL"] = "DEBUG"
    os.environ["RACHER_BACKUP_SKIP_LOADING"] = "1"
    os.environ["RACHER_BACKUP_AMOUNT"] = "0"
    os.environ["RACHER_BACKUP_INTERVAL"] = "120"

    p = subprocess.Popen(cmd)
    time.sleep(1)
    return p


def main():
    a = run(f"{BINARY} -a {HOST_LIST[0]}")

    time.sleep(4)
    others = [
        run(f"{BINARY} join -a {x} -j http://{HOST_LIST[0]}") for x in HOST_LIST[1:]
    ]
    print(a)
    print(others)

    return a.wait()


if __name__ == "__main__":
    main()
