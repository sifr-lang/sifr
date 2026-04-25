from helpers.dsu import UnionFind

from collections import defaultdict

# LeetCode 721: Accounts Merge
# Python version

def accountsMerge(accounts: list[list[str]]) -> list[list[str]]:
    uf = UnionFind(len(accounts))
    emailToAcc = {} # email -> index of acc

    for i, a in enumerate(accounts):
        for e in a[1:]:
            if e in emailToAcc:
                uf.union(i, emailToAcc[e])
            else:
                emailToAcc[e] = i

    emailGroup = defaultdict(list) # index of acc -> list of emails
    for e, i in emailToAcc.items():
        leader = uf.find(i)
        emailGroup[leader].append(e)

    res = []
    for i, emails in emailGroup.items():
        name = accounts[i][0]
        res.append([name] + sorted(emailGroup[i])) # array concat
    return res
def main():
    assert accountsMerge([['John', 'johnsmith@mail.com', 'john_newyork@mail.com'], ['John', 'johnsmith@mail.com', 'john00@mail.com'], ['Mary', 'mary@mail.com'], ['John', 'johnnybravo@mail.com']]) == [['John', 'john00@mail.com', 'john_newyork@mail.com', 'johnsmith@mail.com'], ['Mary', 'mary@mail.com'], ['John', 'johnnybravo@mail.com']]
    assert accountsMerge([['Gabe', 'Gabe0@m.co', 'Gabe3@m.co', 'Gabe1@m.co'], ['Kevin', 'Kevin3@m.co', 'Kevin5@m.co', 'Kevin0@m.co'], ['Ethan', 'Ethan5@m.co', 'Ethan4@m.co', 'Ethan0@m.co'], ['Hanzo', 'Hanzo3@m.co', 'Hanzo1@m.co', 'Hanzo0@m.co'], ['Fern', 'Fern5@m.co', 'Fern1@m.co', 'Fern0@m.co']]) == [['Gabe', 'Gabe0@m.co', 'Gabe1@m.co', 'Gabe3@m.co'], ['Kevin', 'Kevin0@m.co', 'Kevin3@m.co', 'Kevin5@m.co'], ['Ethan', 'Ethan0@m.co', 'Ethan4@m.co', 'Ethan5@m.co'], ['Hanzo', 'Hanzo0@m.co', 'Hanzo1@m.co', 'Hanzo3@m.co'], ['Fern', 'Fern0@m.co', 'Fern1@m.co', 'Fern5@m.co']]

if __name__ == "__main__":
    main()
