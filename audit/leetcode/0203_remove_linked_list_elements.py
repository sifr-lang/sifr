# LeetCode 203: Remove Linked List Elements
# Python version

def removeElements(head: ListNode, val: int) -> ListNode:
    dummy = ListNode(next=head)
    prev, curr = dummy, head
    
    while curr:
        nxt = curr.next
        
        if curr.val == val:
            prev.next = nxt
        else:
            prev = curr
        
        curr = nxt
    return dummy.next



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
