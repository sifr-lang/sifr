# LeetCode 133: Clone Graph
# Python version

def cloneGraph(node: "Node") -> "Node":
    oldToNew = {}

    def dfs(node):
        if node in oldToNew:
            return oldToNew[node]

        copy = Node(node.val)
        oldToNew[node] = copy
        for nei in node.neighbors:
            copy.neighbors.append(dfs(nei))
        return copy

    return dfs(node) if node else None



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
