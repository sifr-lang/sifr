# LeetCode 929: Unique Email Addresses
# Python version

def numUniqueEmails(emails: list[str]) -> int:
    unique_emails: set[str] = set()
    for email in emails:
        local_name, domain_name = email.split('@')
        local_name = local_name.split('+')[0]
        local_name = local_name.replace('.', '')
        email = local_name + '@' + domain_name
        unique_emails.add(email)
    return len(unique_emails)



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
