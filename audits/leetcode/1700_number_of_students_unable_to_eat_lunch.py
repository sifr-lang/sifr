
# LeetCode 1700: Number Of Students Unable To Eat Lunch
# Python version

def countStudents(students: list[int], sandwiches: list[int]) -> int:
    num_of_students_back_in_line = 0
    while num_of_students_back_in_line != len(students):
        curr_student = students.pop(0)
        if curr_student == sandwiches[0]:
            sandwiches.pop(0)
            num_of_students_back_in_line = 0
        else:
            students.append(curr_student)
            num_of_students_back_in_line += 1
    return len(students)



def main():
    assert countStudents([1,1,0,0], [0,1,0,1]) == 0

if __name__ == "__main__":
    main()
