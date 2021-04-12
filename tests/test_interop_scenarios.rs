use openmls::prelude::*;

#[macro_use]
mod utils;

use utils::managed_utils::*;

// The following tests correspond to the interop test scenarios detailed here:
// https://github.com/mlswg/mls-implementations/blob/master/test-scenarios.md
// The tests are conducted for every available ciphersuite, but currently only
// using BasicCredentials. We can change the test setup once #134 is fixed.

fn default_managed_group_config() -> ManagedGroupConfig {
    let handshake_message_format = HandshakeMessageFormat::Plaintext;
    let update_policy = UpdatePolicy::default();
    let callbacks = ManagedGroupCallbacks::default();
    ManagedGroupConfig::new(handshake_message_format, update_policy, 10, 0, callbacks)
}

// # 1:1 join
// A:    Create group
// B->A: KeyPackage
// A->B: Welcome
// ***:  Verify group state
ctest_ciphersuites!(one_to_one_join, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();
    let number_of_clients = 2;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with a random creator.
    let group_id = setup
        .create_group(ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, alice_id) = group.members.first().unwrap().clone();

    // A vector including bob's id.
    let bob_id = setup.random_new_members_for_group(group, 1).unwrap();

    setup
        .add_clients(ActionType::Commit, group, &alice_id, bob_id)
        .expect("Error adding Bob");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # 3-party join
// A: Create group
// B->A: KeyPackage
// A->B: Welcome
// C->A: KeyPackage
// A->B: Add(C), Commit
// A->C: Welcome
// ***:  Verify group state
ctest_ciphersuites!(three_party_join, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 3;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with a random creator.
    let group_id = setup
        .create_group(ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, alice_id) = group.members.first().unwrap().clone();

    // A vector including Bob's id.
    let bob_id = setup.random_new_members_for_group(group, 1).unwrap();

    // Create the add commit and deliver the welcome.
    setup
        .add_clients(ActionType::Commit, group, &alice_id, bob_id)
        .expect("Error adding Bob");

    // A vector including Charly's id.
    let charly_id = setup.random_new_members_for_group(group, 1).unwrap();

    setup
        .add_clients(ActionType::Commit, group, &alice_id, charly_id)
        .expect("Error adding Charly");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Multiple joins at once
// A:    Create group
// B->A: KeyPackage
// C->A: KeyPackage
// A->B: Welcome
// A->C: Welcome
// ***:  Verify group state
ctest_ciphersuites!(multiple_joins, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 3;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with a random creator.
    let group_id = setup
        .create_group(ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, alice_id) = group.members.first().unwrap().clone();

    // A vector including Bob's and Charly's id.
    let bob_charly_id = setup.random_new_members_for_group(group, 2).unwrap();

    // Create the add commit and deliver the welcome.
    setup
        .add_clients(ActionType::Commit, group, &alice_id, bob_charly_id)
        .expect("Error adding Bob and Charly");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Multiple joins at once
// P1:       Create group
// P2->P1:   KeyPackage
// ...
// P100->P1: KeyPackage
// P1->P2:   Welcome
// ...
// P1->P100: Welcome
// ***:  Verify group state
ctest_ciphersuites!(bulk_joins, test(param: CiphersuiteName) {
    let ciphersuite_name = CiphersuiteName::try_from(param).unwrap();
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 100;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);
    setup.create_clients();

    // Create a group with a random creator.
    let group_id = setup
        .create_group(ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, p1) = group.members.first().unwrap().clone();

    // A vector including all other ids.
    let participant_ids = setup.random_new_members_for_group(group, number_of_clients-1).unwrap();

    // Create the add commit and deliver the welcome.
    setup
        .add_clients(ActionType::Commit, group, &p1, participant_ids)
        .expect("Error adding other participants");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// TODO #192, #286, #289: The external join test should go here.

// # Update
// A:    Create group
// B->A: KeyPackage
// A->B: Welcome
// A->B: Update, Commit
// ***:  Verify group state
ctest_ciphersuites!(update, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 2;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with two members. Includes the process of adding Bob.
    let group_id = setup
        .create_random_group(2, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, alice_id) = group.members.first().unwrap().clone();

    // Let Alice create an update with a self-generated KeyPackageBundle.
    setup
        .self_update(ActionType::Commit, group, &alice_id, None)
        .expect("Error self-updating.");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Update large group
// P1:       Create group
// P2->P1:   KeyPackage
// ...
// P100->P1: KeyPackage
// P1->P2:   Welcome
// ...
// P1->P100: Welcome
// P1->P2:   Update, Commit
// ...
// P1->P100: Update, Commit
// ***:  Verify group state
ctest_ciphersuites!(update_large_group, test(param: CiphersuiteName) {
    let ciphersuite_name = CiphersuiteName::try_from(param).unwrap();
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 100;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);
    setup.create_clients();

    // Create a group with two members. Includes the process of adding Bob.
    let group_id = setup
        .create_random_group(number_of_clients, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, p1) = group.members.first().unwrap().clone();

    // Let Alice create an update with a self-generated KeyPackageBundle.
    setup
        .self_update(ActionType::Commit, group, &p1, None)
        .expect("Error self-updating.");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Remove
// A:    Create group
// B->A: KeyPackage
// C->A: KeyPackage
// A->B: Welcome
// A->C: Welcome
// A->B: Remove(B), Commit
// ***:  Verify group state
ctest_ciphersuites!(remove, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 2;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with two members. Includes the process of adding Bob.
    let group_id = setup
        .create_random_group(2, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, alice_id) = group.members.first().unwrap().clone();
    let (_, bob_id) = group.members.last().unwrap().clone();

    // Have alice remove Bob.
    setup
        .remove_clients(ActionType::Commit, group, &alice_id, vec![bob_id])
        .expect("Error removing Bob from the group.");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Remove large group
// P1:       Create group
// P2->P1:   KeyPackage
// ...
// P100->P1: KeyPackage
// P1->P2:   Welcome
// ...
// P1->P100: Welcome
// P1->P2:   Remove(P2), Commit
// ...
// P1->P100: Remove(P2), Commit
// ***:  Verify group state
ctest_ciphersuites!(remove_large_group, test(param: CiphersuiteName) {
    let ciphersuite_name = CiphersuiteName::try_from(param).unwrap();
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 100;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);
    setup.create_clients();

    // Create a group with two members. Includes the process of adding Bob.
    let group_id = setup
        .create_random_group(number_of_clients, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, p1) = group.members.first().unwrap().clone();
    let (_, p2) = group.members.get(1).unwrap().clone();

    // Have alice remove Bob.
    setup
        .remove_clients(ActionType::Commit, group, &p1, vec![p2])
        .expect("Error removing p2 from the group.");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// # Bulk remove
// P1:       Create group
// P2->P1:   KeyPackage
// ...
// P200->P1: KeyPackage
// P1->P2:   Welcome
// ...
// P1->P200: Welcome
// P1->P2:   Remove(P50...P150), Commit
// ...
// P1->P200: Remove(P50...P150), Commit
// ***:  Verify group state
ctest_ciphersuites!(bulk_remove, test(param: CiphersuiteName) {
    let ciphersuite_name = CiphersuiteName::try_from(param).unwrap();
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 200;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);
    setup.create_clients();

    // Create a group with two members. Includes the process of adding Bob.
    let group_id = setup
        .create_random_group(number_of_clients, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let (_, p1) = group.members.first().unwrap().clone();
    let p50_p150 = group.members[49..149].iter().map(|(_, p)| p.clone()).collect();

    // Have alice remove Bob.
    setup
        .remove_clients(ActionType::Commit, group, &p1, p50_p150)
        .expect("Error removing P50...P150 from the group.");

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});

// TODO #141, #284: The external PSK, resumption and re-init tests should go
// here.

// # Large Group, Full Lifecycle
// * Create group
// * Group creator adds the first M members
// * Until group size reaches N members, a randomly-chosen group member adds a
//   new member
// * All members update
// * While the group size is >1, a randomly-chosen group member removes a
//   randomly-chosen other group member
ctest_ciphersuites!(large_group_lifecycle, test(ciphersuite_name: CiphersuiteName) {
    println!("Testing ciphersuite {:?}", ciphersuite_name);
    let ciphersuite = Config::ciphersuite(ciphersuite_name).unwrap();

    let number_of_clients = 50;
    let setup = ManagedTestSetup::new(default_managed_group_config(), number_of_clients);

    // Create a group with all available clients. The process includes creating
    // a one-person group and then adding new members in bunches of up to 5,
    // each bunch by a random group member.
    let group_id = setup
        .create_random_group(number_of_clients, ciphersuite)
        .expect("Error while trying to create group.");
    let mut groups = setup.groups.borrow_mut();
    let group = groups.get_mut(&group_id).unwrap();

    let mut group_members = group.members.clone();

    // Have each member in turn update. In between each update, messages are
    // delivered to each member.
    for (_, member_id) in &group_members {
        setup
            .self_update(ActionType::Commit, group, member_id, None)
            .expect("Error while updating group.")
    }

    while group_members.len() > 1 {
        let remover_id = group.random_group_member();
        let mut target_id = group.random_group_member();
        // Get a random member until it's not the one doing the remove operation.
        while remover_id == target_id {
            target_id = group.random_group_member();
        }
        setup
            .remove_clients(ActionType::Commit, group, &remover_id, vec![target_id])
            .expect("Error while removing group member.");
        group_members = group.members.clone();
    }

    // Check that group members agree on a group state.
    setup.check_group_states(group);
});
