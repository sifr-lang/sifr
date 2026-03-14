from collections import deque

# LeetCode 785: Is Graph Bipartite
# Python version

def isBipartiteBFS(graph: list[list[int]]) -> bool:
    colors = [-1] * len(graph)
    
    for i in range(len(graph)):
        if colors[i] == -1:
            q = deque([i])
            colors[i] = 0
            
            while q:
                node = q.popleft()
                
                for nbh in graph[node]:
                    if colors[nbh] == -1:
                        colors[nbh] = 1 - colors[node]
                        q.append(nbh)
                    elif colors[nbh] == colors[node]:
                        return False  
    
    return True


def isBipartiteDFS(graph: list[list[int]]) -> bool:
    colors = [-1] * len(graph)

    def dfs(node, c):
        colors[node] = c
        
        for nbh in graph[node]:
            if colors[nbh] == -1:
                if not dfs(nbh, 1 - c):
                    return False
            elif colors[nbh] == colors[node]:
                return False
        
        return True

    for i in range(len(graph)):
        if colors[i] == -1:
            if not dfs(i, 0):
                return False
    
    return True


def main():
    assert isBipartiteBFS([[1, 2, 3], [0, 2], [0, 1, 3], [0, 2]]) == False
    assert isBipartiteBFS([[1, 3], [0, 2], [1, 3], [0, 2]]) == True

if __name__ == "__main__":
    main()
