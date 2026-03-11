# LeetCode 150: Evaluate Reverse Polish Notation
# Python version

def evalRPN(tokens: List[str]) -> int:
    stack = []
    for c in tokens:
        if c == "+":
            stack.append(stack.pop() + stack.pop())
        elif c == "-":
            a, b = stack.pop(), stack.pop()
            stack.append(b - a)
        elif c == "*":
            stack.append(stack.pop() * stack.pop())
        elif c == "/":
            a, b = stack.pop(), stack.pop()
            stack.append(int(float(b) / a))
        else:
            stack.append(int(c))
    return stack[0]



def main():
    print(evalRPN(["2","1","+","3","*"]))

if __name__ == "__main__":
    main()
