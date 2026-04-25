
# LeetCode 745: Prefix And Suffix Search
# Python version
class TrieNode:
    def __init__(self):
        self.children = {}  # Dictionary to store child nodes
        self.word = -1  # Store the index of the word at this node
class WordFilter:
    def __init__(self, words: list[str]):
        self.root = TrieNode()
        for index, word in enumerate(words):
            for i in range(len(word) + 1):
                for j in range(len(word) + 1):
                    key = word[i:] + '{' + word[:j]
                    cur = self.root
                    for c in key:
                        if c not in cur.children:
                            cur.children[c] = TrieNode()
                        cur = cur.children[c]
                    cur.word = index  # Store the index of the word at this node
    def f(self, pref: str, suff: str) -> int:
        key = suff + '{' + pref
        cur = self.root
        for c in key:
            if c not in cur.children:
                return -1  # If combination doesn't exist, return -1
            cur = cur.children[c]
        return cur.word  # Return the largest index found for the valid combination

def main():
    obj = WordFilter(['apple'])
    assert obj.f('a', 'e') == 0

if __name__ == "__main__":
    main()
