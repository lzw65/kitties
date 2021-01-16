use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

#[test]
fn creat_test() {
    new_test_ext().execute_with(|| {
		run_to_block(10);
		// balance of account 1 = 2000, reserve 500
		assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));

		// balance of account 5 = 400, reserve 500
		//Q5
		assert_eq!(Kitties::create(Origin::signed(5)), Err(DispatchError::Module { index: 0, error: 3, message: Some("InsufficientBalance") }));
    });
}

#[test]
fn breed_test() {
    new_test_ext().execute_with(|| {
		run_to_block(10);
		// balance of account 1 = 2000, reserve 500
		assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
		assert_eq!(Kitties::create(Origin::signed(2)), Ok(()));
		//Q5
		assert_eq!(Kitties::breed(Origin::signed(3), 1, 1 ), Err(DispatchError::Module { index: 0, error: 2, message: Some("RequireDifferentParent") }));

		assert_eq!(Kitties::breed(Origin::signed(4), 0, 1 ), Ok(()));
		assert_eq!(Kitties::breed(Origin::signed(4), 0, 99 ),Err(DispatchError::Module { index: 0, error: 1, message: Some("InvalidKittyId") }));
    });
}
#[test]
fn transfer_test() {
    new_test_ext().execute_with(|| {
		run_to_block(10);
		// balance of account 1 = 2000, reserve 500
		assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
		//Q5
		assert_eq!(Kitties::transfer(Origin::signed(2), 3, 0 ), Err(DispatchError::Module { index: 0, error: 3, message: Some("NotOwner") }));
		assert_eq!(Kitties::transfer(Origin::signed(1), 5, 0 ), Err(DispatchError::Module { index: 0, error: 3, message: Some("InsufficientBalance") }));
		assert_eq!(Kitties::transfer(Origin::signed(1), 2, 0 ), Ok(()));
		assert_eq!(Kitties::transfer(Origin::signed(1), 3, 0 ), Err(DispatchError::Module { index: 0, error: 3, message: Some("NotOwner")}) );
    });
}