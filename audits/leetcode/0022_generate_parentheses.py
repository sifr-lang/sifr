# LeetCode 22: Generate Parentheses
# Python version

def generateParenthesis(n: int) -> List[str]:
    stack = []
    res = []

    def backtrack(openN, closedN):
        if openN == closedN == n:
            res.append("".join(stack))
            return

        if openN < n:
            stack.append("(")
            backtrack(openN + 1, closedN)
            stack.pop()
        if closedN < openN:
            stack.append(")")
            backtrack(openN, closedN + 1)
            stack.pop()

    backtrack(0, 0)
    return res



def main():
    assert generateParenthesis(3) == ['((()))', '(()())', '(())()', '()(())', '()()()']

if __name__ == "__main__":
    main()
