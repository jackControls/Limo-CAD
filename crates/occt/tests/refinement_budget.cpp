#include "refinement_budget.hpp"
#include <cassert>
#include <limits>

using limo_cad_occt::RefinementBudget;

template <typename Action>
void rejects(Action action, const char* resource) {
  try {
    action();
    throw std::logic_error("Expected refinement resource rejection");
  } catch (const std::runtime_error& error) {
    const std::string message = error.what();
    assert(message.find(resource) != std::string::npos);
    assert(message.find("face 7") != std::string::npos);
  }
}

int main() {
  RefinementBudget budget;
  budget.context = "before standard healing, face 7";
  budget.comparisons = 12;
  budget.samples = 10;
  budget.insertions = 2;
  budget.compare(2, 3);
  budget.sample(2, 3);
  budget.insert(2);
  // The same operation's second repair must not receive a fresh allowance.
  budget.context = "after standard healing, face 7";
  rejects([&] { budget.compare(2, 4); }, "segment comparison");
  assert(budget.comparisons == 6);
  budget.compare(2, 3);
  rejects([&] { budget.compare(1); }, "segment comparison");
  rejects([&] { budget.sample(3, 2); }, "sample allocation");
  assert(budget.samples == 4);
  budget.sample(2, 2);
  rejects([&] { budget.sample(); }, "sample allocation");
  rejects([&] { budget.insert(4096); }, "4096 points per edge");
  assert(budget.insertions == 1);
  budget.insert(4095);
  rejects([&] { budget.insert(2); }, "total insertion");
  // Hostile products cannot wrap into an apparently affordable allowance.
  budget.comparisons = budget.samples = std::numeric_limits<std::size_t>::max();
  rejects([&] { budget.compare(budget.comparisons, 2); }, "segment comparison");
  rejects([&] { budget.sample(budget.samples, 2); }, "sample allocation");
  budget.compare(budget.comparisons, 0);
  budget.sample(budget.samples, 0);
  assert(budget.samples == std::numeric_limits<std::size_t>::max());
}
