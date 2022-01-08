use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, assert_err};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{ Header as _ },
};

use frame_system::pallet_prelude::*;
use frame_support::traits::{ OnInitialize};

fn setup_blocks(blocks: u64) {
	let mut parent_hash = System::parent_hash();

	for i in 1..(blocks + 1) {
		System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
		CollectiveFlip::on_initialize(i);

		let header = System::finalize();
		parent_hash = header.hash();
		System::set_block_number(*header.number());
	}
}

fn events() -> Vec<Event> {
	let evt = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();

	System::reset_events();

	evt
}

//测试创建做法
#[test]
fn create_kitty_test() {
	new_test_ext().execute_with(|| {
		setup_blocks(80);
		// 首先测试Event::Created事件
		assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		assert_eq!(Balances::free_balance(&1), 10000000);
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		let account_id = ensure_signed(Origin::signed(1)).unwrap();
		let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];
		// assert_eq!(events(),
		//     [ 
		// 		Event::System(system::Event::NewAccount { account: 1 }),
		// 		Event::Balances(crate::Event::Endowed { account: 1, free_balance: 10000000 }),
		// 	    Event::KittyModule(crate::Event::Created{0:1, 1:kitty_id}) 
		//     ]
		// );
		System::assert_has_event(Event::KittyModule(crate::Event::Created{0:1, 1:kitty_id}) );


		//测试Error::ExceedMaxKittyOwned
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		assert_err!(KittyModule::create_kitty(Origin::signed(1)), Error::<Test>::ExceedMaxKittyOwned);
		// assert_noop!(KittyModule::create_kitty(Origin::signed(1)), Error::<Test>::ExceedMaxKittyOwned);

		//测试Error::NotEnoughBalance
		assert_noop!(KittyModule::create_kitty( Origin::signed(2)), Error::<Test>::NotEnoughBalance);

	});
}


//设置价格测试
#[test]
fn set_price_test() {
	new_test_ext().execute_with(|| {
		setup_blocks(80);
		assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		assert_eq!(Balances::free_balance(&1), 10000000);
		assert_ok!(Balances::set_balance(Origin::root(), 2, 10000000, 0));
        //是否成功设置价格Event::PriceSet
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		let account_id = ensure_signed(Origin::signed(1)).unwrap();
		let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];
		assert_ok!(KittyModule::set_price(Origin::signed(1), kitty_id, Some(32)));
		System::assert_has_event(Event::KittyModule(crate::Event::PriceSet{0:1, 1:kitty_id, 2:Some(32)}) );
		
		//测试<Error<T>>::KittyNotExist
		let test_h256 = H256([0u8;32]);
		assert_noop!(KittyModule::set_price(Origin::signed(2), test_h256, Some(32)), Error::<Test>::KittyNotExist);
		
		//测试<Error<T>>::NotKittyOwner
		assert_noop!(KittyModule::set_price(Origin::signed(2), kitty_id, Some(32)), Error::<Test>::NotKittyOwner);

	});

}


	//测试贩卖方法
	#[test]
	fn sell_kitty_test() {
		new_test_ext().execute_with(|| {
			setup_blocks(80);
		    assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		    assert_eq!(Balances::free_balance(&1), 10000000);
		    assert_ok!(Balances::set_balance(Origin::root(), 2, 10000000, 0));

		    //测试是否贩卖成功Event::Transferred
		    assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		    let account_id = ensure_signed(Origin::signed(1)).unwrap();
		    let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];
		    let to_account_id = ensure_signed(Origin::signed(2)).unwrap();
		    assert_ok!(KittyModule::sell_kitty(Origin::signed(1), to_account_id, kitty_id));
			System::assert_has_event(Event::KittyModule(crate::Event::Transferred{0:account_id, 1:to_account_id, 2:kitty_id}) );
		});
		
	}


//测试购买方法	
#[test]
fn buy_test(){
	new_test_ext().execute_with(||{
		setup_blocks(80);
		assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		assert_eq!(Balances::free_balance(&1), 10000000);
		assert_ok!(Balances::set_balance(Origin::root(), 2, 10000000, 0));

		//测试<Error<T>>::KittyNotExist
		let test_h256 = H256([0u8;32]);
		assert_noop!(KittyModule::buy_kitty(Origin::signed(1), test_h256, 25),Error::<Test>::KittyNotExist);

		//测试<Error<T>>::BuyerIsKittyOwner
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		let account_id = ensure_signed(Origin::signed(1)).unwrap();
		let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];
		assert_noop!(KittyModule::buy_kitty(Origin::signed(1), kitty_id, 25),Error::<Test>::BuyerIsKittyOwner);
		
		//测试<Error<T>>::KittyNotForSale
		assert_noop!(KittyModule::buy_kitty(Origin::signed(2), kitty_id, 25),Error::<Test>::KittyNotForSale);

		//测试<Error<T>>::KittyBidPriceTooLow
		assert_ok!(KittyModule::set_price(Origin::signed(1), kitty_id, Some(10000)));
		assert_ok!(Balances::set_balance(Origin::root(), 3, 1000, 0));
		assert_noop!(KittyModule::buy_kitty(Origin::signed(4), kitty_id, 25),Error::<Test>::KittyBidPriceTooLow);

        //测试<Error<T>>::NotEnoughBalance
		assert_ok!(KittyModule::set_price(Origin::signed(1), kitty_id, Some(1000)));
		assert_ok!(Balances::set_balance(Origin::root(), 3, 1000, 0));
		assert_noop!(KittyModule::buy_kitty(Origin::signed(3), kitty_id, 10002),Error::<Test>::NotEnoughBalance);

		//测试<Error<T>>::ExceedMaxKittyOwned
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_noop!(KittyModule::buy_kitty(Origin::signed(2), kitty_id, 10002),Error::<Test>::ExceedMaxKittyOwned);

		//测试Event::Bought
		assert_ok!(Balances::set_balance(Origin::root(), 4, 100000, 0));
		let buyer_id = ensure_signed(Origin::signed(4)).unwrap();
		let seller_id = ensure_signed(Origin::signed(1)).unwrap();
		assert_ok!(KittyModule::buy_kitty(Origin::signed(4), kitty_id, 10002));
		System::assert_has_event(Event::KittyModule(crate::Event::Bought{0:buyer_id, 1:seller_id, 2:kitty_id, 3:10002}) );
	});
	
}

//测试Transfer
#[test]
fn transfer_test(){
	new_test_ext().execute_with(||{
		setup_blocks(80);
		assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		assert_eq!(Balances::free_balance(&1), 10000000);
		assert_ok!(Balances::set_balance(Origin::root(), 2, 10000000, 0));
		assert_ok!(Balances::set_balance(Origin::root(), 3, 10000000, 0));

		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		let account_id = ensure_signed(Origin::signed(1)).unwrap();
		let account_id_2 = ensure_signed(Origin::signed(2)).unwrap();
		let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];

		//测试<Error<T>>::NotKittyOwner
		assert_noop!(KittyModule::transfer(Origin::signed(2), account_id, kitty_id), Error::<Test>::NotKittyOwner);

        //测试<Error<T>>::TransferToSelf
		assert_noop!(KittyModule::transfer(Origin::signed(1), account_id, kitty_id), Error::<Test>::TransferToSelf);

		//测试<Error<T>>::ExceedMaxKittyOwned
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
		assert_noop!(KittyModule::transfer(Origin::signed(1), account_id_2, kitty_id), Error::<Test>::ExceedMaxKittyOwned);

		//测试Event::Transferred
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		let kitty_id_2 = KittyModule::kitties_owned(&account_id).into_inner()[0];
		let account_id_3 = ensure_signed(Origin::signed(3)).unwrap();
		assert_ok!(KittyModule::transfer(Origin::signed(1), account_id_3, kitty_id));
		System::assert_has_event(Event::KittyModule(crate::Event::Transferred{0:account_id, 1:account_id_3, 2:kitty_id_2}) );
	});
}

//培育方法测试
#[test]
fn breed_test(){
	new_test_ext().execute_with(||{
		setup_blocks(80);
		assert_ok!(Balances::set_balance(Origin::root(), 1, 10000000, 0));
		assert_ok!(Balances::set_balance(Origin::root(), 2, 10000000, 0));
		assert_eq!(Balances::free_balance(&1), 10000000);

        assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));

		//测试<Error<T>>::NotKittyOwner
		let account_id = ensure_signed(Origin::signed(1)).unwrap();
		let kitty_id = KittyModule::kitties_owned(&account_id).into_inner()[0];
		let kitty_id_2 = KittyModule::kitties_owned(&account_id).into_inner()[1];
		assert_noop!(KittyModule::breed_kitty(Origin::signed(2), kitty_id ,kitty_id_2),Error::<Test>::NotKittyOwner);
		
		//测试培育是否成功
		assert_ok!(KittyModule::breed_kitty(Origin::signed(1), kitty_id ,kitty_id_2));

	});


}