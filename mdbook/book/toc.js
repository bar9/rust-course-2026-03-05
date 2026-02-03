// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Day 1: Foundations &amp; Ownership</li><li class="chapter-item expanded "><a href="day1/01_setup.html"><strong aria-hidden="true">1.</strong> Course Introduction &amp; Setup</a></li><li class="chapter-item expanded "><a href="day1/02_fundamentals.html"><strong aria-hidden="true">2.</strong> Rust Fundamentals</a></li><li class="chapter-item expanded "><a href="day1/03_structs_enums.html"><strong aria-hidden="true">3.</strong> Structs, Enums, and Methods</a></li><li class="chapter-item expanded "><a href="day1/04_ownership.html"><strong aria-hidden="true">4.</strong> Memory Model &amp; Ownership</a></li><li class="chapter-item expanded "><a href="day1/05_smart_pointers.html"><strong aria-hidden="true">5.</strong> Ownership Patterns &amp; Smart Pointers</a></li><li class="chapter-item expanded affix "><li class="part-title">Day 2: Type System &amp; Error Handling</li><li class="chapter-item expanded "><a href="day2/06_collections.html"><strong aria-hidden="true">6.</strong> Collections Deep Dive</a></li><li class="chapter-item expanded "><a href="day2/07_traits.html"><strong aria-hidden="true">7.</strong> Traits &amp; Polymorphism</a></li><li class="chapter-item expanded "><a href="day2/08_generics.html"><strong aria-hidden="true">8.</strong> Generics &amp; Type Safety</a></li><li class="chapter-item expanded "><a href="day2/09_pattern_matching.html"><strong aria-hidden="true">9.</strong> Enums &amp; Pattern Matching</a></li><li class="chapter-item expanded "><a href="day2/10_error_handling.html"><strong aria-hidden="true">10.</strong> Error Handling Deep Dive</a></li><li class="chapter-item expanded "><a href="day2/11_iterators.html"><strong aria-hidden="true">11.</strong> Iterators &amp; Functional Programming</a></li><li class="chapter-item expanded "><a href="day2/12_modules_visibility.html"><strong aria-hidden="true">12.</strong> Modules &amp; Visibility</a></li><li class="chapter-item expanded affix "><li class="part-title">Day 3: ESP32-C3 Temperature Monitor - From Sensor to System</li><li class="chapter-item expanded "><a href="day3/13_hardware_hello.html"><strong aria-hidden="true">13.</strong> Hardware Hello - ESP32-C3 Basics</a></li><li class="chapter-item expanded "><a href="day3/14_embedded_foundations.html"><strong aria-hidden="true">14.</strong> Embedded Foundations - no_std from the Start</a></li><li class="chapter-item expanded "><a href="day3/15_testing_embedded.html"><strong aria-hidden="true">15.</strong> Testing Embedded Code</a></li><li class="chapter-item expanded "><a href="day3/16_data_communication.html"><strong aria-hidden="true">16.</strong> Data &amp; Communication</a></li><li class="chapter-item expanded "><a href="day3/17_integration_deployment.html"><strong aria-hidden="true">17.</strong> Integration &amp; Deployment</a></li><li class="chapter-item expanded "><a href="day3/18_complete_system.html"><strong aria-hidden="true">18.</strong> Performance Optimization &amp; Power Management</a></li><li class="chapter-item expanded affix "><li class="part-title">Day 4: Enterprise Transition - From C++/.NET to Production Rust</li><li class="chapter-item expanded "><a href="transfer/19_cargo.html"><strong aria-hidden="true">19.</strong> Cargo &amp; Dependency Management</a></li><li class="chapter-item expanded "><a href="transfer/20_codecov.html"><strong aria-hidden="true">20.</strong> Code Coverage with CodeCov</a></li><li class="chapter-item expanded "><a href="transfer/21_macros.html"><strong aria-hidden="true">21.</strong> Macros &amp; Code Generation</a></li><li class="chapter-item expanded "><a href="transfer/22_unsafe_ffi.html"><strong aria-hidden="true">22.</strong> Unsafe Rust &amp; FFI with Bindgen</a></li><li class="chapter-item expanded "><a href="transfer/23_embedded_hal.html"><strong aria-hidden="true">23.</strong> Embedded HAL - Hardware Register Access &amp; Volatile Memory</a></li><li class="chapter-item expanded "><a href="transfer/24_async_concurrency.html"><strong aria-hidden="true">24.</strong> Async and Concurrency</a></li><li class="chapter-item expanded "><a href="transfer/25_rust_patterns.html"><strong aria-hidden="true">25.</strong> Rust Patterns</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
