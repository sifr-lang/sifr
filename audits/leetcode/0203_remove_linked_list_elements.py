
# LeetCode 203: Remove Linked List Elements
# Python version
from helpers.list_node import ListNode, list_node_to_string
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
    assert list_node_to_string(removeElements(ListNode(1, ListNode(2, ListNode(6, ListNode(3, ListNode(4, ListNode(5, ListNode(6, None))))))), 6)) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))))
    assert removeElements(None, 1) == None
    assert removeElements(ListNode(7, ListNode(7, ListNode(7, ListNode(7, None)))), 7) == None

if __name__ == "__main__":
    main()
