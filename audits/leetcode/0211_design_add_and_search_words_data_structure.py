
# LeetCode 211: Design Add And Search Words Data Structure
# Python version
class TrieNode:
    def __init__(self):
        self.children = {}  # a : TrieNode
        self.word = False
class WordDictionary:
    def __init__(self):
        self.root = TrieNode()
    def addWord(self, word: str) -> None:
        cur = self.root
        for c in word:
            if c not in cur.children:
                cur.children[c] = TrieNode()
            cur = cur.children[c]
        cur.word = True
    def search(self, word: str) -> bool:
        def dfs(j, root):
            cur = root
            for i in range(j, len(word)):
                c = word[i]
                if c == ".":
                    for child in cur.children.values():
                        if dfs(i + 1, child):
                            return True
                    return False
                else:
                    if c not in cur.children:
                        return False
                    cur = cur.children[c]
            return cur.word
        return dfs(0, self.root)

def main():
    obj = WordDictionary()
    obj.addWord('bad')
    obj.addWord('dad')
    obj.addWord('mad')
    assert obj.search('pad') == False
    assert obj.search('bad') == True
    assert obj.search('.ad') == True
    assert obj.search('b..') == True

if __name__ == "__main__":
    main()
