# LeetCode 682: Baseball Game
# Python version

def calPoints(operations: List[str]) -> int:
    
    score_stack = []
    
    for o in operations:
        
        # it is +, D, or C
        # if stack isn't of sufficient length, then operation is voided
        if o == "+" and len(score_stack) >= 2:
            summed = score_stack[-2] + score_stack[-1]
            score_stack.append(summed)
            
        elif o == "D" and len(score_stack) >= 1:
            doubled = score_stack[-1] * 2
            score_stack.append(doubled)
            
        elif o == "C" and len(score_stack) >= 1:
            score_stack.pop() 
            
        else: 
            score_stack.append(int(o))

    return sum(score_stack)


def main():
    print(calPoints(["5","2","C","D","+"]))

if __name__ == "__main__":
    main()
