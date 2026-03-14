from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 929: Unique Email Addresses
# Python version

def numUniqueEmails(emails: list[str]) -> int:
    unique_emails: set[str] = set()
    for email in emails:
        local_name, domain_name = email.split('@')
        local_name = local_name.split('+')[0]
        local_name = local_name.replace('.', '')
        email = local_name + '@' + domain_name
        unique_emails.add(email)
    return len(unique_emails)



def main():
    assert numUniqueEmails(['test.email+alex@leetcode.com', 'test.e.mail+bob.cathy@leetcode.com', 'testemail+david@lee.tcode.com']) == 2
    assert numUniqueEmails(['a@leetcode.com', 'b@leetcode.com', 'c@leetcode.com']) == 3

if __name__ == "__main__":
    main()
