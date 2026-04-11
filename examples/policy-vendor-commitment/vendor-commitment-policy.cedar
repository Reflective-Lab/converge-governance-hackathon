// Vendor commitment policy for the hackathon governance server.
//
// A procurement user or governed agent may propose, validate, promote, or
// commit a vendor recommendation depending on authority, amount, and approval.

permit(principal, action == Action::"propose", resource)
when {
  principal.authority == "advisory" ||
  principal.authority == "supervisory" ||
  principal.authority == "participatory" ||
  principal.authority == "sovereign"
};

permit(principal, action == Action::"validate", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "participatory" ||
  principal.authority == "sovereign"
};

permit(principal, action == Action::"promote", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "sovereign"
};

permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "sovereign"
};

permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "supervisory" &&
  context.human_approval_present == true &&
  context.required_gates_met == true
};

forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.human_approval_present == false
};

permit(principal, action == Action::"advance_phase", resource)
when {
  (principal.authority == "supervisory" || principal.authority == "sovereign") &&
  context.required_gates_met == true
};

forbid(principal, action == Action::"propose", resource)
when {
  principal.authority == "advisory" &&
  context.commitment_type == "spend" &&
  context.amount > 10000
};
