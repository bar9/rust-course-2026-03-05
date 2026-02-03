// Chapter 13: Modules and Visibility Exercise Solution

// =============================================================================
// Exercise: Library Management System
// =============================================================================

pub mod books {
    pub struct Book {
        pub title: String,
        pub author: String,
        pub isbn: String,
        available: bool, // Private field
    }

    impl Book {
        pub fn new(title: String, author: String, isbn: String) -> Self {
            Book {
                title,
                author,
                isbn,
                available: true,
            }
        }

        pub fn is_available(&self) -> bool {
            self.available
        }

        pub(super) fn set_available(&mut self, available: bool) {
            self.available = available;
        }
    }
}

pub mod members {
    pub struct Member {
        pub id: u32,
        pub name: String,
        pub email: String,
        active: bool, // Private field
    }

    impl Member {
        pub fn new(id: u32, name: String, email: String) -> Self {
            Member {
                id,
                name,
                email,
                active: true,
            }
        }

        pub fn is_active(&self) -> bool {
            self.active
        }

    }
}

pub mod loans {

    pub struct Loan {
        book_isbn: String,
        member_id: u32,
        due_date: String,
    }

    impl Loan {
        pub fn new(book_isbn: String, member_id: u32, due_date: String) -> Self {
            Loan {
                book_isbn,
                member_id,
                due_date,
            }
        }

        pub fn book_isbn(&self) -> &str {
            &self.book_isbn
        }

        pub fn member_id(&self) -> u32 {
            self.member_id
        }

        pub fn due_date(&self) -> &str {
            &self.due_date
        }
    }
}

pub mod library {
    use super::loans::Loan;

    // Re-export types for convenience
    pub use super::books::Book;
    pub use super::members::Member;

    pub struct Library {
        pub books: Vec<Book>,
        pub members: Vec<Member>,
        loans: Vec<Loan>, // Private
    }

    impl Library {
        pub fn new() -> Self {
            Library {
                books: Vec::new(),
                members: Vec::new(),
                loans: Vec::new(),
            }
        }

        pub fn add_book(&mut self, book: Book) {
            self.books.push(book);
        }

        pub fn add_member(&mut self, member: Member) {
            self.members.push(member);
        }

        pub fn checkout_book(&mut self, isbn: &str, member_id: u32, due_date: String) -> Result<(), String> {
            // Find book
            let book = self.books.iter_mut()
                .find(|b| b.isbn == isbn)
                .ok_or("Book not found")?;

            if !book.is_available() {
                return Err("Book is not available".to_string());
            }

            // Find member
            let member = self.members.iter()
                .find(|m| m.id == member_id)
                .ok_or("Member not found")?;

            if !member.is_active() {
                return Err("Member is not active".to_string());
            }

            // Create loan
            book.set_available(false);
            let loan = Loan::new(isbn.to_string(), member_id, due_date);
            self.loans.push(loan);

            Ok(())
        }

        pub fn return_book(&mut self, isbn: &str) -> Result<(), String> {
            // Find and remove loan
            let loan_index = self.loans.iter()
                .position(|loan| loan.book_isbn() == isbn)
                .ok_or("No active loan found for this book")?;

            self.loans.remove(loan_index);

            // Find book and mark as available
            let book = self.books.iter_mut()
                .find(|b| b.isbn == isbn)
                .ok_or("Book not found")?;

            book.set_available(true);

            Ok(())
        }

        pub fn active_loans(&self) -> usize {
            self.loans.len()
        }

        pub fn get_loan(&self, isbn: &str) -> Option<&Loan> {
            self.loans.iter().find(|loan| loan.book_isbn() == isbn)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use library::*;

    #[test]
    fn test_book_creation() {
        let book = Book::new(
            "The Rust Programming Language".to_string(),
            "Steve Klabnik".to_string(),
            "978-1718500440".to_string(),
        );

        assert_eq!(book.title, "The Rust Programming Language");
        assert_eq!(book.author, "Steve Klabnik");
        assert_eq!(book.isbn, "978-1718500440");
        assert!(book.is_available());
    }

    #[test]
    fn test_member_creation() {
        let member = Member::new(
            1,
            "Alice".to_string(),
            "alice@example.com".to_string(),
        );

        assert_eq!(member.id, 1);
        assert_eq!(member.name, "Alice");
        assert_eq!(member.email, "alice@example.com");
        assert!(member.is_active());
    }

    #[test]
    fn test_loan_creation() {
        let loan = loans::Loan::new("123456789".to_string(), 1, "2024-01-15".to_string());

        assert_eq!(loan.book_isbn(), "123456789");
        assert_eq!(loan.member_id(), 1);
        assert_eq!(loan.due_date(), "2024-01-15");
    }

    #[test]
    fn test_library_creation() {
        let library = Library::new();
        assert_eq!(library.books.len(), 0);
        assert_eq!(library.members.len(), 0);
        assert_eq!(library.active_loans(), 0);
    }

    #[test]
    fn test_library_add_book() {
        let mut library = Library::new();
        let book = Book::new(
            "Rust Book".to_string(),
            "Author".to_string(),
            "123456789".to_string(),
        );

        library.add_book(book);
        assert_eq!(library.books.len(), 1);
        assert_eq!(library.books[0].title, "Rust Book");
    }

    #[test]
    fn test_library_add_member() {
        let mut library = Library::new();
        let member = Member::new(
            1,
            "Alice".to_string(),
            "alice@example.com".to_string(),
        );

        library.add_member(member);
        assert_eq!(library.members.len(), 1);
        assert_eq!(library.members[0].name, "Alice");
    }

    #[test]
    fn test_successful_checkout() {
        let mut library = Library::new();

        // Add book and member
        library.add_book(Book::new(
            "Rust Book".to_string(),
            "Author".to_string(),
            "123456789".to_string(),
        ));
        library.add_member(Member::new(
            1,
            "Alice".to_string(),
            "alice@example.com".to_string(),
        ));

        // Checkout book
        let result = library.checkout_book("123456789", 1, "2024-01-15".to_string());
        assert!(result.is_ok());
        assert_eq!(library.active_loans(), 1);

        // Book should no longer be available
        assert!(!library.books[0].is_available());

        // Should be able to find the loan
        let loan = library.get_loan("123456789");
        assert!(loan.is_some());
        assert_eq!(loan.unwrap().member_id(), 1);
    }

    #[test]
    fn test_checkout_book_not_found() {
        let mut library = Library::new();
        library.add_member(Member::new(1, "Alice".to_string(), "alice@example.com".to_string()));

        let result = library.checkout_book("nonexistent", 1, "2024-01-15".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Book not found");
    }

    #[test]
    fn test_checkout_member_not_found() {
        let mut library = Library::new();
        library.add_book(Book::new("Book".to_string(), "Author".to_string(), "123".to_string()));

        let result = library.checkout_book("123", 999, "2024-01-15".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Member not found");
    }

    #[test]
    fn test_checkout_book_not_available() {
        let mut library = Library::new();

        library.add_book(Book::new("Book".to_string(), "Author".to_string(), "123".to_string()));
        library.add_member(Member::new(1, "Alice".to_string(), "alice@example.com".to_string()));
        library.add_member(Member::new(2, "Bob".to_string(), "bob@example.com".to_string()));

        // First checkout succeeds
        assert!(library.checkout_book("123", 1, "2024-01-15".to_string()).is_ok());

        // Second checkout fails
        let result = library.checkout_book("123", 2, "2024-01-15".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Book is not available");
    }

    #[test]
    fn test_successful_return() {
        let mut library = Library::new();

        library.add_book(Book::new("Book".to_string(), "Author".to_string(), "123".to_string()));
        library.add_member(Member::new(1, "Alice".to_string(), "alice@example.com".to_string()));

        // Checkout and return
        assert!(library.checkout_book("123", 1, "2024-01-15".to_string()).is_ok());
        assert_eq!(library.active_loans(), 1);

        assert!(library.return_book("123").is_ok());
        assert_eq!(library.active_loans(), 0);
        assert!(library.books[0].is_available());
    }

    #[test]
    fn test_return_book_no_loan() {
        let mut library = Library::new();
        library.add_book(Book::new("Book".to_string(), "Author".to_string(), "123".to_string()));

        let result = library.return_book("123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active loan found for this book");
    }

    #[test]
    fn test_module_visibility() {
        let book = books::Book::new("Title".to_string(), "Author".to_string(), "ISBN".to_string());

        // Public fields are accessible
        assert_eq!(book.title, "Title");
        assert_eq!(book.author, "Author");
        assert_eq!(book.isbn, "ISBN");

        // But private field is not directly accessible
        // book.available; // This would not compile

        // Must use public method
        assert!(book.is_available());
    }

    #[test]
    fn test_re_exported_types() {
        // Test that we can use Book and Member through the library module
        let book = library::Book::new("Test".to_string(), "Author".to_string(), "123".to_string());
        let member = library::Member::new(1, "Test".to_string(), "test@example.com".to_string());

        assert_eq!(book.title, "Test");
        assert_eq!(member.name, "Test");
    }

    #[test]
    fn test_complex_library_operations() {
        let mut library = Library::new();

        // Add multiple books and members
        library.add_book(Book::new("Book1".to_string(), "Author1".to_string(), "111".to_string()));
        library.add_book(Book::new("Book2".to_string(), "Author2".to_string(), "222".to_string()));
        library.add_book(Book::new("Book3".to_string(), "Author3".to_string(), "333".to_string()));

        library.add_member(Member::new(1, "Alice".to_string(), "alice@example.com".to_string()));
        library.add_member(Member::new(2, "Bob".to_string(), "bob@example.com".to_string()));

        // Perform multiple checkouts
        assert!(library.checkout_book("111", 1, "2024-01-15".to_string()).is_ok());
        assert!(library.checkout_book("222", 2, "2024-01-16".to_string()).is_ok());

        assert_eq!(library.active_loans(), 2);

        // Return one book
        assert!(library.return_book("111").is_ok());
        assert_eq!(library.active_loans(), 1);

        // Check that the right book is available again
        assert!(library.books.iter().find(|b| b.isbn == "111").unwrap().is_available());
        assert!(!library.books.iter().find(|b| b.isbn == "222").unwrap().is_available());
        assert!(library.books.iter().find(|b| b.isbn == "333").unwrap().is_available());
    }
}