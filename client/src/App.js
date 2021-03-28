import React, { Component } from 'react';
import { connect } from 'react-redux';
import * as C from './Const'

import Following from './Following';
import MyFardels from './MyFardels';
import NewFardel from './NewFardel';
import UnpackedFardels from './UnpackedFardels';
import Profile from './Profile';
import HandleSearchField from './HandleSearchField';
import HandleView from './HandleView';
import About from './About';

import { SigningCosmWasmClient } from 'secretjs';
import { setKeplrEnabled,
         setScrtAuthorized,
         setSecretClient,
         setDrawerMenuSelection,
         setViewingKey,
         queryGetHandle } from './store/Actions';
import { getKeplr } from './utils/keplr';

//Material-UI
import AppBar from '@material-ui/core/AppBar';
import Grid from '@material-ui/core/Grid';
import Button from '@material-ui/core/Button';
import CssBaseline from '@material-ui/core/CssBaseline';
import Dialog from '@material-ui/core/Dialog';
import DialogActions from '@material-ui/core/DialogActions';
import DialogContent from '@material-ui/core/DialogContent';
import DialogContentText from '@material-ui/core/DialogContentText';
import DialogTitle from '@material-ui/core/DialogTitle';
import Divider from '@material-ui/core/Divider';
import TextField from '@material-ui/core/TextField';
import Drawer from '@material-ui/core/Drawer';
import Toolbar from '@material-ui/core/Toolbar';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import ListItemIcon from '@material-ui/core/ListItemIcon';
import ListItemText from '@material-ui/core/ListItemText';
import PublicTwoToneIcon from '@material-ui/icons/PublicTwoTone';
import FaceTwoToneIcon from '@material-ui/icons/FaceTwoTone';
import CardGiftcardTwoToneIcon from '@material-ui/icons/CardGiftcardTwoTone';
import GroupAddTwoToneIcon from '@material-ui/icons/GroupAddTwoTone';
import InfoTwoToneIcon from '@material-ui/icons/InfoTwoTone';
import ImportContactsTwoToneIcon from '@material-ui/icons/ImportContactsTwoTone';
import BackupTwoToneIcon from '@material-ui/icons/BackupTwoTone';

import { withStyles } from '@material-ui/core/styles';

const secretLCD = process.env.REACT_APP_SecretLCD;

const drawerWidth = 260;

const useStyles = theme => ({
  root: {
    display: 'flex',
  },
  appBar: {
    background : '#202225',
    zIndex: theme.zIndex.drawer + 1,
  },
  drawer: {
    width: drawerWidth,
    flexShrink: 0,
  },
  drawerPaper: {
    width: drawerWidth,
  },
  drawerContainer: {
    overflow: 'auto',
  },
  content: {
    flexGrow: 1,
    padding: theme.spacing(3),
  },
});

const chainId = process.env.REACT_APP_ChainId;

class App extends Component {

  constructor(props) {
    super(props);
    this.state = { 
      loginDialogOpen: false,
      loginKey: '',
    };
  }

  async componentDidMount() {
    const { scrtAuthorized, setScrtAuthorized, setKeplrEnabled, setSecretClient } = this.props;

    const keplrCheckPromise = new Promise ( (resolve, reject) => {
      const keplrCheckInterval = setInterval(async () => {
        let isKeplrWallet = !!window.keplr && !!window.getOfflineSigner && !! window.getEnigmaUtils;
        setKeplrEnabled(true);
        if (isKeplrWallet) {
          clearInterval(keplrCheckInterval);
          resolve();
        }
      }, 1000);
    });
 
    keplrCheckPromise.then( async () => {
      try {
        await window.keplr.enable(chainId);
        const offlineSigner = window.getOfflineSigner(chainId);
        const enigmaUtils = window.getEnigmaUtils(chainId);
        const accounts = await offlineSigner.getAccounts();
        const cosmJS = new SigningCosmWasmClient(
          secretLCD,
          accounts[0].address,
          offlineSigner,
          enigmaUtils
        );
        if (!scrtAuthorized) {
          setScrtAuthorized(true);
        }
        setSecretClient(cosmJS);
      } catch (error) {
        if (scrtAuthorized) {
          setScrtAuthorized(false);
        }
        console.log('Keplr login error', error);
      }
    });
  }

  componentDidUpdate(prevProps) {
    const { keplrEnabled } = this.props;
    if (prevProps.keplrEnabled !== keplrEnabled) {
      window.addEventListener("keplr_keystorechange", () => {
        console.log("Key store in Keplr is changed. You may need to refetch the account info.")
      })
      console.log(window);
    }
  }

  handleDialogCloseButtonClick() {
    this.setState({
      loginKey: '',
      loginDialogOpen: false,
    })
  }

  handleDialogLoginButtonClick() {
    const { queryGetHandle, handle, setViewingKey } = this.props;
    queryGetHandle(this.state.loginKey);
    setViewingKey(this.state.loginKey);
    this.setState({
      loginKey: '',
      loginDialogOpen: false,
    });
  }

  handleDialogKeyInput(event) {
    this.setState({
      loginKey: event.target.value,
    });
  }

  handleLoginButton() {
    const { scrtAuthorized } = this.props;

    if (scrtAuthorized) {
      this.setState({
        loginDialogOpen: true,
      })
    }
  }

  renderMain() {
    const { drawerMenuSelection } = this.props;

    switch(drawerMenuSelection) {
      case C.DRAWER_MENU_VIEW_HANDLE:
        return <HandleView />;
      case C.DRAWER_MENU_FOLLOWING:
        return <Following />;
      case C.DRAWER_MENU_UNPACKED_FARDELS:
        return <UnpackedFardels />;
      case C.DRAWER_MENU_NEW_FARDEL:
        return <NewFardel />;
      //case C.DRAWER_MENU_MY_FARDELS:
      //  return <MyFardels />;
      case C.DRAWER_MENU_PROFILE:
        return <Profile />;
      case C.DRAWER_MENU_ABOUT:
        return <About />;
      default:
        return <MyFardels />;
    }    
  }

  renderLogin() {
    const { handle, scrtAuthorized, loggedIn, setDrawerMenuSelection } = this.props;

    if (scrtAuthorized && loggedIn) {
      return (
        <Grid item>
          <div>
            <Button raised color="primary" onClick={() => setDrawerMenuSelection(C.DRAWER_MENU_PROFILE)}>
              {"@"+handle}
            </Button>
          </div>
        </Grid>
      );
    } else if (scrtAuthorized) {
      return (
        <Grid item>
          <div>
            <Button raised color="primary" onClick={() => this.handleLoginButton()}>
              Login
            </Button>
          </div>
        </Grid>
      );
    } else {
      return <div />;
    }
  }

  render() {
    const { classes,
            scrtAuthorized, 
            drawerMenuSelection, 
            keplrEnabled,
            viewedHandle,
            setDrawerMenuSelection,
            loggedIn,
            handle,
          } = this.props;

    return (
      <div className={classes.root}>
        <CssBaseline />
        <AppBar className={classes.appBar}>
          <Toolbar>
            <Grid 
              justify="space-between"
              container
              spacing={24}
            >
              <Grid item>
                <img src="/fardels_sm.png" alt="fardels" />
              </Grid>
              { this.renderLogin() }
            </Grid>
          </Toolbar>
        </AppBar>
        <Drawer
          className={classes.drawer}
          variant="permanent"
          classes={{
            paper: classes.drawerPaper,
          }}
        >
          <Toolbar />
          <div className={classes.drawerContainer}>
            <List>
              <ListItem
                selected={drawerMenuSelection===C.DRAWER_MENU_VIEW_HANDLE}
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_VIEW_HANDLE)}
              >
                  <ListItemIcon><PublicTwoToneIcon /></ListItemIcon>
                  <ListItemText primary={"Explore"} />
              </ListItem>
              <ListItem>
                <HandleSearchField />
              </ListItem>
              <ListItem 
                button 
                key="Following" 
                selected={drawerMenuSelection===C.DRAWER_MENU_FOLLOWING} 
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_FOLLOWING)} 
              >
                  <ListItemIcon><GroupAddTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="Following" />
              </ListItem>
          {/**
              <ListItem 
                button 
                key="Unpacked Fardels"
                selected={drawerMenuSelection===C.DRAWER_MENU_UNPACKED_FARDELS}
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_UNPACKED_FARDELS)} 
              >
                  <ListItemIcon><ImportContactsTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="Unpacked" />
              </ListItem>
          **/}
              <Divider />
              <ListItem 
                button 
                key="Carry New" 
                selected={drawerMenuSelection===C.DRAWER_MENU_NEW_FARDEL} 
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_NEW_FARDEL)} 
              >
                  <ListItemIcon><BackupTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="Carry New" />
              </ListItem>
          {/**
              <ListItem 
                button 
                key="My Fardels"
                selected={drawerMenuSelection===C.DRAWER_MENU_MY_FARDELS}
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_MY_FARDELS)} 
              >
                  <ListItemIcon><CardGiftcardTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="My Fardels" />
              </ListItem>
          **/}
              <ListItem 
                button 
                key="My Profile"
                selected={drawerMenuSelection===C.DRAWER_MENU_PROFILE} 
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_PROFILE)} 
              >
                  <ListItemIcon><FaceTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="My Profile" />
              </ListItem>
              <Divider />
              <ListItem 
                button 
                key="About"
                selected={drawerMenuSelection===C.DRAWER_MENU_ABOUT} 
                onClick={(e) => setDrawerMenuSelection(C.DRAWER_MENU_ABOUT)} 
              >
                  <ListItemIcon><InfoTwoToneIcon /></ListItemIcon>
                  <ListItemText primary="About" />
              </ListItem>
            </List>
            <i>there is that in this fardel will make him scratch his beard.</i>
          </div>
        </Drawer>
        <main className={classes.content}>
          <Toolbar />
          { this.renderMain() }
        </main>
        <Dialog 
          open={this.state.loginDialogOpen} 
          onClose={() => this.handleDialogCloseButtonClick()} 
          aria-labelledby="start-following-dialog-title">
          <DialogTitle id="start-following-dialog-title">Login</DialogTitle>
          <DialogContent>
            <DialogContentText>
              Enter your viewing key to login to your handle. Go to My Profile to set a new viewing key or change your handle (costs gas).
            </DialogContentText>
            <TextField
              autoFocus
              margin="dense"
              id="viewing key"
              label="Viewing key"
              type="password"
              fullWidth
              value={this.state.loginKey}
              onChange={(e) => this.handleDialogKeyInput(e)}
            />
          </DialogContent>
          <DialogActions>
            <Button onClick={() => this.handleDialogCloseButtonClick()} color="primary">
              Cancel
            </Button>
            <Button onClick={() => this.handleDialogLoginButtonClick()} color="primary">
              Login
            </Button>
          </DialogActions>
        </Dialog>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    scrtAuthorized: state.scrtAuthorized,
    secretClient: state.secretClient,
    drawerMenuSelection: state.drawerMenuSelection,
    viewedHandle: state.viewedHandle,
    loggedIn: state.loggedIn,
    handle: state.handle,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    setViewingKey: (key) => { dispatch(setViewingKey(key)) },
    queryGetHandle: (key) => { dispatch(queryGetHandle(key)) },
    setKeplrEnabled: (keplrEnabled) => { dispatch(setKeplrEnabled(keplrEnabled)) },
    setScrtAuthorized: (scrtAuthorized)   => { dispatch(setScrtAuthorized(scrtAuthorized)) },
    setSecretClient: (secretClient) => { dispatch(setSecretClient(secretClient)) },
    setDrawerMenuSelection: (drawerMenuSelection) => { dispatch(setDrawerMenuSelection(drawerMenuSelection)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(App));
