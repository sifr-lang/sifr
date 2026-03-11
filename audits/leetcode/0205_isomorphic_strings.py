# LeetCode 205: Isomorphic Strings
# Python version

def isIsomorphic(s: str, t: str) -> bool:
    mapST, mapTS = {}, {}

    for c1, c2 in zip(s, t):
        if (c1 in mapST and mapST[c1] != c2) or (c2 in mapTS and mapTS[c2] != c1):
            return False
        mapST[c1] = c2
        mapTS[c2] = c1

    return True


def main():
    print(isIsomorphic("egg", "add"))
    print(isIsomorphic("foo", "bar"))

if __name__ == "__main__":
    main()
