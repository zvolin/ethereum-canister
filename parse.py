#!/usr/bin/env python

import re
import sys
from pathlib import Path

import pandas


def read_records(logfile):
    with open(logfile) as f:
        logs = f.read()

    if "erc20" in logfile:
        method = "erc20_balance_of"
        variant = logfile.split("of-")[1].split(".")[0]
    elif "erc721" in logfile:
        method = "erc721_owner_of"
        variant = logfile.split("721-")[1].split(".")[0]
    elif "setup" in logfile:
        method = "setup"
        variant = logfile.split("setup-")[1].split(".")[0]
    elif "advance" in logfile:
        method = "advance"
        variant = ""
    elif "get-block-number" in logfile:
        method = "get_block_number"
        variant = ""
    else:
        print("unknown method")
        sys.exit(1)

    last_mem_size = None
    last_balance = None
    mem_size = None
    balance = None
    req_count = 0
    requests = []
    responses = []
    internal_balance = None
    internal_instructions = None

    records = []

    for line in logs.splitlines():
        if "Balance:" in line:
            balance = int(re.search(r"([0-9_]+) Cycles", line)[1])

            if internal_balance is not None:
                records.append({
                    "method": method,
                    "variant": variant,
                    "prev_balance": last_balance,
                    "balance": balance,
                    "prev_memsize": last_mem_size,
                    "memsize": mem_size,
                    "http outcalls": req_count,
                    "request_sizes": requests,
                    "response_sizes": responses,
                    "payments [cycles]": internal_balance,
                    "instructions count": internal_instructions,
                })

            last_mem_size = mem_size
            last_balance = balance
            mem_size = None
            balance = None
            req_count = 0
            requests = []
            responses = []
            internal_balance = None
            internal_instructions = None

        elif "Memory Size:" in line:
            mem_size = int(re.search(r"Nat\(([0-9]+)\)", line)[1])
        elif "GET" in line or "POST" in line:
            req_count += 1
        elif "resp size: " in line:
            responses.append(int(re.search(r"resp size: ([0-9]+)b", line)[1]))
        elif "request size: " in line:
            requests.append(int(re.search(r"request size: ([0-9]+)", line)[1]))
        elif "instructions:" in line:
            if method.replace('_', ' ') in line.lower():
                internal_instructions = int(re.search(r"ons: ([0-9]+)", line)[1])
        elif "balance diff:" in line:
            if method.replace('_', ' ') in line.lower():
                internal_balance = int(re.search(r"iff: ([0-9]+)", line)[1])

    return records

all_records = []
for log in Path(".").glob("run-*"):
    all_records += read_records(str(log))

df = pandas.DataFrame.from_dict(all_records)
df.to_csv("cycle-costs.csv")

df["call cost [cycles]"] = df["prev_balance"] - df["balance"]
df["mem usage diff [b]"] = df["memsize"] - df["prev_memsize"]

summary = df.groupby(["method", "variant"]).max()[[
    "call cost [cycles]",
    "mem usage diff [b]",
    "payments [cycles]",
    "instructions count",
    "https outcalls",
]]

print(summary)
